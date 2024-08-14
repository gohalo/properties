#![no_main]

use libfuzzer_sys::fuzz_target;
use properties::{Properties, WriteOption};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut buff = Vec::new();
        let mut prop = Properties::new();
        prop.set("Hello", s);
        let _ = prop.store(&mut buff, &WriteOption::default());
    }
});
