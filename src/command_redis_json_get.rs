use crate::rejson::REDIS_JSON_TYPE;
use jsonpath_lib::select;
use redis_module::{Context, NextArg, RedisError, RedisResult, RedisString, RedisValue};
use serde_json::{json, to_string, Value};
use std::collections::HashMap;

// TODO: support for formatting parameters
pub fn cmd(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1);

    let key = args.next_arg()?;
    let paths = args.map(|s| s).collect::<Vec<RedisString>>();

    let key_ptr = ctx.open_key_writable(&key);
    let key_value = key_ptr.get_value::<Value>(&REDIS_JSON_TYPE)?;
    let jsn = match key_value {
        Some(v) => v,
        None => return Ok(RedisValue::Null),
    };

    let res = match paths.len() {
        0 => match select(jsn, "$")?.pop() {
            Some(v) => Ok(json!(v)),
            None => return Ok(RedisValue::Null),
        },
        1 => match select(jsn, &paths[0].to_string()) {
            Ok(v) => Ok(json!(v)),
            Err(_) => return Ok(RedisValue::Null),
        },
        _ => {
            let m = paths
                .iter()
                .map(|p| p.to_string())
                .map(|p| (p.clone(), select(jsn, &p)))
                .filter(|(_, r)| r.is_ok())
                .map(|(p, r)| (p, r.unwrap()))
                .collect::<HashMap<String, Vec<&Value>>>();
            Ok(json!(m))
        }
    };

    return match res {
        Ok(v) => Ok(RedisValue::StringBuffer(
            to_string(&v).unwrap().as_bytes().to_vec(),
        )),
        Err(e) => Err(RedisError::String(e)),
    };
}
