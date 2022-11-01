#![no_main]

use jsonpath_rs;

use arbitrary::Arbitrary;
use arbitrary_json::ArbitraryValue;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    path: String,
    value: ArbitraryValue,
}

fuzz_target!(|data: FuzzInput| {
    let _ = jsonpath_rs::get(&data.path, &data.value);
});
