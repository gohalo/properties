#![no_main]

use libfuzzer_sys::fuzz_target;

use properties::Properties;

fuzz_target!(|data: &[u8]| {
    let _ = Properties::new().load(data);
});
