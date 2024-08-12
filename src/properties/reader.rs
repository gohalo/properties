use std::io::Read;

use super::Properties;
use super::Result;

fn load_convert(data: &[u8]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut idx = 0;
    let end = data.len();
    let mut c: u8;

    while idx < end {
        c = data[idx];
        if c == b'\\' {
            idx = idx + 1;
            c = match data[idx] {
                b't' => b'\t',
                b'r' => b'\r',
                b'n' => b'\n',
                b'f' => b'\x0c',
                c => c,
            };
        }
        result.push(c);
        idx = idx + 1;
    }
    return result;
}

// Read in a "logical line" from an reader, skip all comment and
// blank lines and filter out those leading whitespace characters
// (\u0020, \u0009 and \u000c) from beginning of a "natural line".
// Method returns the char length of the "logical line" and stores
// the line in "line" field.
struct LineReader {
    buff: [u8; 8 * 1024],
    line: Vec<u8>,
    limit: usize,
    offset: usize,
}

impl LineReader {
    fn new() -> Self {
        Self {
            buff: [0u8; 8 * 1024],
            line: Vec::with_capacity(1024),
            limit: 0,
            offset: 0,
        }
    }

    fn read_line<R: Read>(&mut self, mut reader: R) -> Result<&Vec<u8>> {
        let mut c: u8;
        let mut skip_lf = false;
        let mut skip_white_space = true;
        let mut is_new_line = true;
        let mut is_comment_line = false;
        let mut preceding_backslash = false;
        let mut appended_line_begin = false;

        self.line.clear();
        loop {
            if self.offset >= self.limit {
                self.limit = reader.read(&mut self.buff)?;
                if self.limit <= 0 {
                    if is_comment_line {
                        self.line.clear();
                    }
                    if preceding_backslash {
                        self.line.pop();
                    }
                    return Ok(&self.line);
                }
                //log::info!("Read {} bytes", self.limit);
            }
            c = self.buff[self.offset];
            self.offset = self.offset + 1;

            //log::info!(
            //    "Handle char {}, lf={} white={} newline={} comment={} backslash={}",
            //    c as char,
            //    skip_lf,
            //    skip_white_space,
            //    is_new_line,
            //    is_comment_line,
            //    preceding_backslash
            //);

            if skip_lf {
                skip_lf = false;
                if c == b'\n' {
                    continue;
                }
            }

            if skip_white_space {
                if c == b' ' || c == b'\t' || c == b'\x0c' {
                    continue;
                }
                if !appended_line_begin && (c == b'\r' || c == b'\n') {
                    continue;
                }
                skip_white_space = false;
                appended_line_begin = false;
            }

            if is_new_line {
                is_new_line = false;
                if c == b'#' || c == b'!' {
                    is_comment_line = true;
                    continue;
                }
            }

            if c != b'\n' && c != b'\r' {
                self.line.push(c);
                if c == b'\\' {
                    preceding_backslash = !preceding_backslash;
                } else {
                    preceding_backslash = false;
                }
            } else {
                // new natural line
                if is_comment_line {
                    is_comment_line = false;
                    is_new_line = true;
                    skip_white_space = true;
                    self.line.clear();
                    continue;
                }

                if preceding_backslash {
                    self.line.pop();
                    // skip the leading whitespace characters in following line
                    skip_white_space = true;
                    appended_line_begin = true;
                    preceding_backslash = false;
                    if c == b'\r' {
                        skip_lf = true;
                    }
                } else {
                    return Ok(&self.line);
                }
            }
        }
    }
}

impl Properties {
    pub fn load<R: Read>(&mut self, mut reader: R) -> Result<()> {
        let mut lr = LineReader::new();
        loop {
            match lr.read_line(&mut reader) {
                Ok(l) => {
                    if l.len() == 0 {
                        return Ok(());
                    }
                    //log::info!("=====> Read line {}", std::str::from_utf8(l).unwrap());

                    let mut key_len = 0;
                    let mut value_start = 0;
                    let mut backslash = false;
                    let mut has_sep = false;
                    let line = l.as_slice();
                    let limit = l.len();

                    while key_len < limit {
                        let c = line[key_len];
                        if (c == b'=' || c == b':') && !backslash {
                            value_start = key_len + 1;
                            has_sep = true;
                            break;
                        } else if (c == b' ' || c == b'\t' || c == b'\x0c') && !backslash {
                            value_start = key_len + 1;
                            break;
                        }
                        if c == b'\\' {
                            backslash = !backslash;
                        } else {
                            backslash = false;
                        }
                        key_len = key_len + 1;
                    }

                    while value_start < limit {
                        let c = line[value_start];
                        if c != b' ' && c != b'\t' && c != b'\x0c' {
                            if !has_sep && (c == b'=' || c == b':') {
                                has_sep = true;
                            } else {
                                break;
                            }
                        }
                        value_start = value_start + 1;
                    }

                    //log::info!(
                    //    "Got key={} value={}",
                    //    String::from_utf8(l[..key_len].to_vec()).unwrap(),
                    //    String::from_utf8(l[value_start..].to_vec()).unwrap(),
                    //);

                    let key: String = String::from_utf8(load_convert(&l[..key_len])).unwrap();
                    let val: String = String::from_utf8(load_convert(&l[value_start..])).unwrap();
                    self.data.lock().unwrap().insert(key, val);
                }
                Err(e) => return Err(e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Properties;

    #[test]
    fn parse_properties() {
        let cases = vec![
            ("", vec![]),
            ("a0=b", vec![("a0", "b")]),
            ("a0 \t\x0c b", vec![("a0", "b")]),
            ("a0\tb", vec![("a0", "b")]),
            ("a0\x0cb", vec![("a0", "b")]),
            ("a0:b", vec![("a0", "b")]),
            ("a1 = \t\x0cb", vec![("a1", "b")]),
            ("a2=b\\:b", vec![("a2", "b:b")]),
            ("a3=b\\\n   c, d ", vec![("a3", "bc, d ")]),
            ("a3=\\\\\\\ncd", vec![("a3", "\\cd")]),
            ("a4=d\\", vec![("a4", "d")]),
            ("a5=\\th\\re\\nl\\fl", vec![("a5", "\th\re\nl\x0cl")]),
            ("a6=\u{4f60}\u{597d}\u{1f310}", vec![("a6", "你好🌐")]),
            ("\\!a=\\#b", vec![("!a", "#b")]),
            (
                "a0=b\na1=c\\\nd=e\ra2=d\r\n#comment1\n#comment2\\\na3=e\\\r\n#asy\n \n#comment4",
                vec![("a0", "b"), ("a1", "cd=e"), ("a2", "d"), ("a3", "e#asy")],
            ),
        ];
        for &(input, ref r) in &cases {
            let mut prop = Properties::new();
            match prop.load(input.as_bytes()) {
                Ok(_) => {
                    if r.len() != prop.len() {
                        panic!("invalid items, expect={} got={}", r.len(), prop.len());
                    }
                    for l in r {
                        match prop.get(&l.0) {
                            Some(val) => {
                                if val.ne(&l.1) {
                                    panic!("invalid key {}, expect={} got={}", l.0, l.1, val);
                                }
                            }
                            None => panic!("key {} doesn't exist", l.0),
                        }
                    }
                }
                Err(e) => panic!("Load properties failed, {}", e),
            }
        }
    }
}
