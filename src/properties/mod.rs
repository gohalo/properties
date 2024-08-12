mod reader;
mod writer;

pub use writer::WriteOption;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use encoding_rs::{Encoder, Encoding, WINDOWS_1252};

pub struct PropertiesError {
    desc: String,
    cause: Option<Box<dyn Error>>,
}

impl PropertiesError {
    fn new<S: Into<String>>(desc: S, cause: Option<Box<dyn Error>>) -> Self {
        Self {
            desc: desc.into(),
            cause,
        }
    }
}

impl Display for PropertiesError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.desc)
    }
}

impl From<io::Error> for PropertiesError {
    fn from(e: io::Error) -> Self {
        PropertiesError::new("io error", Some(Box::new(e)))
    }
}

type Result<T> = std::result::Result<T, PropertiesError>;

pub struct Properties {
    encoder: Encoder,
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl Properties {
    pub fn new() -> Self {
        Self::with_encoding(WINDOWS_1252)
    }

    pub fn with_encoding(encoding: &'static Encoding) -> Self {
        Self {
            encoder: encoding.new_encoder(),
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn len(&mut self) -> usize {
        self.data.lock().unwrap().len()
    }

    pub fn set(&mut self, key: &str, value: &str) {
        let mut data = self.data.lock().unwrap();
        data.insert(key.to_string(), value.to_string());
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        let data = self.data.lock().unwrap();
        match data.get(key) {
            Some(val) => Some(val.clone()),
            None => None,
        }
    }
}
