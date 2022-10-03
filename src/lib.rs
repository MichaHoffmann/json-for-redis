#[macro_use]
extern crate redis_module;

mod command_redis_json_get;
mod command_redis_json_set;
mod command_redis_json_type;
mod rejson;
mod util;

use rejson::REDIS_JSON_TYPE;

redis_module! {
    name: "json",
    version: 1,
    data_types: [REDIS_JSON_TYPE],
    commands: [
        ["json.get", command_redis_json_get::cmd, "readonly", 0, 0, 0],
        ["json.set", command_redis_json_set::cmd, "write deny-oom", 0, 0, 0],
        ["json.type", command_redis_json_type::cmd, "readonly", 0, 0, 0],
    ],
}
