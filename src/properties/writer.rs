use std::io::Write;

use super::Properties;
use super::Result;

pub const CR: &[u8] = b"\r";
pub const LF: &[u8] = b"\n";
pub const CRLF: &[u8] = b"\r\n";

pub struct WriteOption {
    comments: String,
    escape_unicode: bool,
    line_ending: &'static [u8],
}

impl WriteOption {
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

fn escape_unicode(target: Vec<u8>) {}

fn save_comment(data: &String) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    return result;
}

fn save_convert(data: &String, escape_space: bool, escape_unicode: bool) -> Vec<u8> {
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
                    let mut ch: usize = c as usize;
                    result.push(b'\\');
                    result.push(b'u');
                    if ch >= 0x10000 {
                        ch = (((c as usize - 0x10000) >> 10) & 0x3FF) + 0xD800;
                        result.push(hex(ch >> 12));
                        result.push(hex(ch >> 8));
                        result.push(hex(ch >> 4));
                        result.push(hex(ch));

                        ch = (c as usize & 0x03FF) + 0xDC00;
                        result.push(b'\\');
                        result.push(b'u');
                    }
                    result.push(hex(ch >> 12));
                    result.push(hex(ch >> 8));
                    result.push(hex(ch >> 4));
                    result.push(hex(ch));
                }
                _ => {
                    let _ = result.write(&bytes[i..i + c.len_utf8()]);
                }
            }
        }
    }
    return result;
}

impl Properties {
    pub fn store<W: Write>(&mut self, mut writer: W, opt: &WriteOption) -> Result<()> {
        if opt.comments.len() > 0 {
            writer.write(&save_comment(&opt.comments))?;
        }

        let data = self.data.lock().unwrap();
        for (k, v) in data.iter() {
            log::info!("key={} value={}", k, v);
            let key = save_convert(k, true, opt.escape_unicode);
            let val = save_convert(v, false, opt.escape_unicode);

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
    use super::{Properties, WriteOption, CRLF};

    #[test]
    fn normal() {
        let cases = vec![
            ("a0", "b", "a0=b\n"),
            (" a 1 ", " b c ", "\\ a\\ 1\\ =\\ b c \n"),
            ("a2", "\\b", "a2=\\\\b\n"),
            ("a3", "\t\n\r\x0c=:#!b", "a3=\\t\\n\\r\\f\\=\\:\\#\\!b\n"),
            ("a4", "你好🌐", "a4=你好🌐\n"),
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
            ("a0", "你好🌐", "a0=\\u4F60\\u597D\\uD83C\\uDF10\r\n"),
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
}
