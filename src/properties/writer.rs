use std::io::Write;

use super::Properties;
use super::Result;

pub const CR: &[u8] = b"\r";
pub const LF: &[u8] = b"\n";
pub const CRLF: &[u8] = b"\r\n";

pub struct WriteOption {
    comments: String,
    line_ending: &'static [u8],
}

impl Default for WriteOption {
    fn default() -> Self {
        Self {
            comments: String::new(),
            line_ending: LF,
        }
    }
}

fn save_convert(data: &[u8], escape_space: bool) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut idx = 0;
    let end = data.len();
    let mut c: u8;

    while idx < end {
        c = data[idx];
        if c > 61 && c < 127 {
            if c == b'\\' {
                result.push(b'\\');
                result.push(b'\\');
            } else {
                result.push(c);
            }
        } else {
            match c {
                b' ' => {
                    if idx == 0 || escape_space {
                        result.push(b'\\');
                    }
                    result.push(b' ');
                }
                b'\t' => {
                    result.push(b'\\');
                    result.push(b't');
                }
                b'\n' => {
                    result.push(b'\\');
                    result.push(b'n');
                }
                b'\r' => {
                    result.push(b'\\');
                    result.push(b'r');
                }
                b'\x0c' => {
                    result.push(b'\\');
                    result.push(b'f');
                }
                b'=' => {
                    result.push(b'\\');
                    result.push(b'=');
                }
                b':' => {
                    result.push(b'\\');
                    result.push(b':');
                }
                b'#' => {
                    result.push(b'\\');
                    result.push(b'#');
                }
                b'!' => {
                    result.push(b'\\');
                    result.push(b'!');
                }
                _ => result.push(c),
            }
        }
        idx = idx + 1;
    }
    return result;
}

impl Properties {
    pub fn store<W: Write>(&mut self, mut writer: W, opt: &WriteOption) -> Result<()> {
        if opt.comments.len() > 0 {
            println!("Comments {}", opt.comments);
        }

        let data = self.data.lock().unwrap();
        for (k, v) in data.iter() {
            log::info!("key={} value={}", k, v);
            let key = save_convert(k.as_bytes(), true);
            let val = save_convert(v.as_bytes(), false);

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
    use super::{Properties, WriteOption};

    #[test]
    fn store_properties() {
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
}
