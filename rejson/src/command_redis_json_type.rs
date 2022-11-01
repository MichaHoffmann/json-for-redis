use crate::jsonpath::get;
use crate::rejson::REDIS_JSON_TYPE;
use redis_module::{Context, NextArg, RedisResult, RedisString, RedisValue};
use serde_json::Value;

pub fn cmd(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1);

    let key = args.next_arg()?;
    let path = match args.next_string() {
        Ok(v) => v,
        Err(_) => "$".to_string(),
    };
    args.done()?;

    let key_ptr = ctx.open_key_writable(&key);
    let key_value = key_ptr.get_value::<Value>(&REDIS_JSON_TYPE)?;
    let jsn = match key_value {
        Some(v) => v,
        None => return Ok(RedisValue::Null),
    };
    let matches = match get(&path, jsn) {
        Ok(v) => v,
        Err(_) => return Ok(RedisValue::Null),
    };

    if path == "$" {
        let v = unsafe { matches.get_unchecked(0) };
        return Ok(RedisValue::StringBuffer(json_type(v).as_bytes().to_vec()));
    }
    return Ok(RedisValue::Array(
        matches
            .iter()
            .map(|v| json_type(v).as_bytes().to_vec())
            .map(RedisValue::StringBuffer)
            .collect(),
    ));
}

fn json_type(m: &Value) -> String {
    let r = {
        if m.is_f64() {
            "number"
        } else if m.is_i64() || m.is_u64() {
            "integer"
        } else if m.is_string() {
            "string"
        } else if m.is_boolean() {
            "boolean"
        } else if m.is_null() {
            "null"
        } else if m.is_array() {
            "array"
        } else if m.is_object() {
            "object"
        } else {
            "undefined"
        }
    };
    r.to_owned()
}
