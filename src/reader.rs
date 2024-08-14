use std::io::{Read, Write};

use super::{Properties, PropertiesError, Result};

fn decode_unicode(data: &[u8]) -> Result<u32> {
    let mut val: u32 = 0;
    for &v in data {
        match v {
            b'0'..=b'9' => {
                val = (val << 4) + (v - b'0') as u32;
            }
            b'a'..=b'f' => {
                val = (val << 4) + 10 + (v - b'a') as u32;
            }
            b'A'..=b'F' => {
                val = (val << 4) + 10 + (v - b'A') as u32;
            }
            _ => {
                return Err(PropertiesError::new(format!(
                    "parse unicode failed, invalid char '{}'",
                    v as char
                )))
            }
        }
    }
    Ok(val)
}

fn load_convert(data: &[u8]) -> Result<Vec<u8>> {
    let mut result: Vec<u8> = Vec::new();
    let mut idx = 0;
    let end = data.len();
    let mut c: u8;
    let mut val: u32;

    while idx < end {
        c = data[idx];
        //println!("-------> {} {}", idx, c as char);
        if c == b'\\' {
            // must have one more byte, check line reader
            idx = idx + 1;
            c = data[idx];
            if c == b'u' {
                if idx + 5 > end {
                    return Err(PropertiesError::new(
                        "invalid escape unicode, at least 4 bytes",
                    ));
                }

                val = decode_unicode(&data[idx + 1..idx + 5])?;
                idx = idx + 5;
                if val >= 0xD800 && val <= 0xDBFF {
                    if idx + 6 > end {
                        return Err(PropertiesError::new(
                            "invalid surrogates escape unicode, at least 6 bytes",
                        ));
                    }

                    if data[idx] != b'\\' || data[idx + 1] != b'u' {
                        return Err(PropertiesError::new(format!(
                            "got lead surrogates without trail {:04X}",
                            val
                        )));
                    }

                    idx = idx + 2;
                    let v = decode_unicode(&data[idx..idx + 4])?;
                    if v < 0xDC00 || v > 0xDFFF {
                        return Err(PropertiesError::new(format!(
                            "invalid trail surrogates, '{:04X}' should between [0xDC00, 0xDFFF]",
                            v
                        )));
                    }
                    val = ((val - 0xD800) << 10) + v - 0xDC00 + 0x10000;
                    idx = idx + 4;
                }

                // should not failed here
                if let Some(ch) = char::from_u32(val) {
                    result.write(ch.to_string().as_bytes())?;
                }
            } else {
                match c {
                    b't' => result.push(b'\t'),
                    b'r' => result.push(b'\r'),
                    b'n' => result.push(b'\n'),
                    b'f' => result.push(b'\x0c'),
                    _ => result.push(c),
                };
                idx = idx + 1;
            }
        } else {
            result.push(c);
            idx = idx + 1;
        }
    }
    return Ok(result);
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
            }
            c = self.buff[self.offset];
            self.offset = self.offset + 1;

            //println!(
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
                    preceding_backslash = false;
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
                    //println!("Got new line: {}", String::from_utf8(l.to_vec()).unwrap());

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

                    //println!(
                    //    "Got key={} value={}",
                    //    String::from_utf8(l[..key_len].to_vec()).unwrap(),
                    //    String::from_utf8(l[value_start..].to_vec()).unwrap(),
                    //);

                    let key: String = String::from_utf8(load_convert(&l[..key_len])?.to_vec())?;
                    let val: String = String::from_utf8(load_convert(&l[value_start..])?.to_vec())?;
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
    fn abnormal() {
        let cases = vec![
            ("key=\\u4xyz", "parse unicode failed, invalid char"),
            ("key=\\u4f6", "invalid escape unicode, at least 4 bytes"),
            (
                "key=\\ud83c\\more data",
                "got lead surrogates without trail",
            ),
            (
                "key=\\ud83c\\ue00",
                "invalid surrogates escape unicode, at least 6 bytes",
            ),
            ("key=\\ud83c\\uda00", "invalid trail surrogates"),
            ("key=\\ud83c\\ue000", "invalid trail surrogates"),
        ];
        for (input, err) in &cases {
            let mut prop = Properties::new();
            match prop.load(input.as_bytes()) {
                Ok(_) => {
                    if let Some(_) = prop.get("key") {
                        panic!("Item should not exist")
                    }
                    panic!("Load properties should failed")
                }
                Err(e) => {
                    let msg = format!("{}", e);
                    assert!(msg.starts_with(err), "{}", e);
                }
            }
        }
    }

    #[test]
    fn normal() {
        let cases = vec![
            ("", vec![]),
            ("a0=b", vec![("a0", "b")]),
            ("a0 \t\x0c b", vec![("a0", "b")]),
            ("a0\tb", vec![("a0", "b")]),
            ("a0\x0cb", vec![("a0", "b")]),
            ("a0:b", vec![("a0", "b")]),
            ("a0=\\", vec![("a0", "")]),
            ("a0=\\b", vec![("a0", "b")]),
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

    #[test]
    fn unicode() {
        let cases = vec![
            (
                "a0=\\u4F60\\u597D\\u00A9\\uD83C\\uDF10\n",
                vec![("a0", "你好©🌐")],
            ),
            (
                "a0=\\u4f60\\u597d\\u00a9\\ud83c\\udf10\n",
                vec![("a0", "你好©🌐")],
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
