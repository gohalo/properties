#![no_main]

use libfuzzer_sys::fuzz_target;
use properties::{Properties, WriteOption};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut buff = Vec::new();
        let mut prop = Properties::new();
        let mut opt = WriteOption::default();
        opt.comments(s.to_string());
        prop.set("Hello", "World");
        let _ = prop.store(&mut buff, &WriteOption::default());
    }
});
