#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate jsonpath_rs;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        jsonpath_rs::parser::parse(s);
    }
});
