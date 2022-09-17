#[macro_use]
extern crate redis_module;

mod commands;
mod rejson;

use commands::{redis_json_get, redis_json_set};
use rejson::REDIS_JSON_TYPE;

redis_module! {
    name: "json",
    version: 1,
    data_types: [REDIS_JSON_TYPE],
    commands: [
        ["json.get", redis_json_get, "readonly", 0, 0, 0],
        ["json.set", redis_json_set, "write deny-oom", 0, 0, 0],
    ],
}
