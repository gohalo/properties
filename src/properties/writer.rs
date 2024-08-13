use std::io::Write;

use super::{Properties, Result};

pub const CR: &[u8] = b"\r";
pub const LF: &[u8] = b"\n";
pub const CRLF: &[u8] = b"\r\n";

pub struct WriteOption {
    comments: String,
    escape_unicode: bool,
    line_ending: &'static [u8],
}

impl WriteOption {
    pub fn comments(&mut self, val: String) -> &Self {
        self.comments = val;
        self
    }

    pub fn escape_unicode(&mut self, val: bool) -> &Self {
        self.escape_unicode = val;
        self
    }

    pub fn line_ending(&mut self, val: &'static [u8]) -> &Self {
        self.line_ending = val;
        self
    }
}

impl Default for WriteOption {
    fn default() -> Self {
        Self {
            comments: String::new(),
            escape_unicode: false,
            line_ending: LF,
        }
    }
}

fn hex(c: usize) -> u8 {
    const CHARS: [u8; 16] = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E',
        b'F',
    ];
    return CHARS[c & 0x0F];
}

fn do_escape_unicode(target: &mut Vec<u8>, c: usize) {
    let mut ch = c;
    target.push(b'\\');
    target.push(b'u');
    if ch >= 0x10000 {
        ch = (((c - 0x10000) >> 10) & 0x3FF) + 0xD800;
        target.push(hex(ch >> 12));
        target.push(hex(ch >> 8));
        target.push(hex(ch >> 4));
        target.push(hex(ch));

        ch = (c & 0x03FF) + 0xDC00;
        target.push(b'\\');
        target.push(b'u');
    }
    target.push(hex(ch >> 12));
    target.push(hex(ch >> 8));
    target.push(hex(ch >> 4));
    target.push(hex(ch));
}

fn save_comment(
    data: &String,
    escape_unicode: bool,
    line_ending: &'static [u8],
) -> Result<Vec<u8>> {
    let mut result: Vec<u8> = Vec::new();
    let mut last: usize = 0;
    let bytes = data.as_bytes();
    let end = bytes.len();
    let mut indices = data.char_indices();

    result.push(b'#');
    while let Some((mut index, c)) = indices.next() {
        match c {
            '\r' | '\n' => {
                result.write(&bytes[last..index])?;
                result.write(line_ending)?;
                if c == '\r' && index + 1 < end && bytes[index + 1] == b'\n' {
                    indices.next();
                    index = index + 1;
                }
                last = index + 1;
                if last < end && bytes[last] != b'#' {
                    result.push(b'#');
                }
            }
            _ if c > '\u{007f}' && escape_unicode => {
                result.write(&bytes[last..index])?;
                do_escape_unicode(&mut result, c as usize);
                last = index + c.len_utf8();
            }
            _ => {}
        }
    }
    if last < end {
        result.write(&bytes[last..])?;
    }
    result.write(line_ending)?;
    return Ok(result);
}

fn save_convert(data: &String, escape_space: bool, escape_unicode: bool) -> Result<Vec<u8>> {
    let bytes = data.as_bytes();
    let mut result: Vec<u8> = Vec::new();

    for (i, c) in data.char_indices() {
        if c > 61 as char && c < 127 as char {
            if c == '\\' {
                result.push(b'\\');
                result.push(b'\\');
            } else {
                result.push(c as u8);
            }
        } else {
            match c {
                ' ' => {
                    if i == 0 || escape_space {
                        result.push(b'\\');
                    }
                    result.push(b' ');
                }
                '\t' => {
                    result.push(b'\\');
                    result.push(b't');
                }
                '\n' => {
                    result.push(b'\\');
                    result.push(b'n');
                }
                '\r' => {
                    result.push(b'\\');
                    result.push(b'r');
                }
                '\x0c' => {
                    result.push(b'\\');
                    result.push(b'f');
                }
                '=' | ':' | '#' | '!' => {
                    result.push(b'\\');
                    result.push(c as u8);
                }
                c if escape_unicode && (c < '\u{0020}' || c > '\u{007e}') => {
                    do_escape_unicode(&mut result, c as usize);
                }
                _ => {
                    result.write(&bytes[i..i + c.len_utf8()])?;
                }
            }
        }
    }
    return Ok(result);
}

impl Properties {
    pub fn store<W: Write>(&mut self, mut writer: W, opt: &WriteOption) -> Result<()> {
        if opt.comments.len() > 0 {
            writer.write(&save_comment(
                &opt.comments,
                opt.escape_unicode,
                opt.line_ending,
            )?)?;
        }

        let data = self.data.lock().unwrap();
        for (k, v) in data.iter() {
            let key = save_convert(k, true, opt.escape_unicode)?;
            let val = save_convert(v, false, opt.escape_unicode)?;

            writer.write(&key)?;
            writer.write(b"=")?;
            writer.write(&val)?;
            writer.write(opt.line_ending)?;
        }
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Properties, WriteOption, CR, CRLF};

    #[test]
    fn normal() {
        let cases = vec![
            ("a0", "b", "a0=b\n"),
            (" a 1 ", " b c ", "\\ a\\ 1\\ =\\ b c \n"),
            ("a2", "\\b", "a2=\\\\b\n"),
            ("a3", "\t\n\r\x0c=:#!b", "a3=\\t\\n\\r\\f\\=\\:\\#\\!b\n"),
            ("a4", "你好©🌐", "a4=你好©🌐\n"),
        ];
        for &(key, val, expected) in &cases {
            let mut buff = Vec::new();
            let mut prop = Properties::new();
            prop.set(key, val);
            if let Err(e) = prop.store(&mut buff, &WriteOption::default()) {
                panic!("store properties failed, {}", e);
            }
            let actual = String::from_utf8(buff).unwrap();
            if actual != expected {
                panic!("unexpected result, got {} expect {}", actual, expected);
            }
        }
    }

    #[test]
    fn escape_unicode() {
        let cases = vec![
            (
                "a0",
                "你好©🌐",
                "a0=\\u4F60\\u597D\\u00A9\\uD83C\\uDF10\r\n",
            ),
            ("a1", "\x01Hello", "a1=\\u0001Hello\r\n"),
        ];
        let mut opt = WriteOption::default();
        opt.escape_unicode(true);
        opt.line_ending(CRLF);

        for &(key, val, expected) in &cases {
            let mut buff = Vec::new();
            let mut prop = Properties::new();
            prop.set(key, val);
            if let Err(e) = prop.store(&mut buff, &opt) {
                panic!("store properties failed, {}", e);
            }
            let actual = String::from_utf8(buff).unwrap();
            if actual != expected {
                panic!("unexpected result, got {} expect {}", actual, expected);
            }
        }
    }

    #[test]
    fn comments() {
        let cases = vec![(
            true,
            "Hello\r\n你好©🌐\nWorld",
            "a0",
            "b",
            "#Hello\r#\\u4F60\\u597D\\u00A9\\uD83C\\uDF10\r#World\ra0=b\r",
        )];
        let mut opt = WriteOption::default();
        opt.line_ending(CR);

        for &(escape, comment, key, val, expected) in &cases {
            let mut buff = Vec::new();
            let mut prop = Properties::new();
            opt.escape_unicode(escape);
            opt.comments(comment.to_string());
            prop.set(key, val);
            if let Err(e) = prop.store(&mut buff, &opt) {
                panic!("store properties failed, {}", e);
            }
            let actual = String::from_utf8(buff).unwrap();
            if actual != expected {
                panic!("unexpected result, got {} expect {}", actual, expected);
            }
        }
    }
}
