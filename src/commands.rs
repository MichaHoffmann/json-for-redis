use redis_module::raw::KeyType;
use redis_module::{Context, NextArg, RedisError, RedisResult, RedisString, RedisValue, REDIS_OK};

use jsonpath_rust::JsonPathQuery;
use serde_json::{from_str, json, to_string, Value};

use std::collections::HashMap;

use crate::rejson::*;

// TODO: support for formatting parameters
pub fn redis_json_get(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
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
        1 => jsn.clone().path(&paths[0].to_string()),
        _ => {
            let mut m = HashMap::new();
            for path in paths {
                let pp = path.to_string();
                match jsn.clone().path(&pp) {
                    Ok(v) => m.insert(pp, v),
                    Err(_) => None,
                };
            }
            Ok(json!(m))
        }
    };

    return match res {
        Ok(v) => Ok(RedisValue::SimpleString(to_string(&v).unwrap())),
        Err(e) => Err(RedisError::String(e)),
    };
}

// TODO: support for updating existing keys
pub fn redis_json_set(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1);

    let key = args.next_arg()?;
    let path = args.next_string()?;
    let val = args.next_string()?;
    // TODO: let nx_or_xx = args.next_string()?;
    args.done()?;

    let jsn: Value = from_str(&val)?;

    // TODO: we only allow setting a nonexisting key at the root for now
    if path != "$" {
        return Err(RedisError::Str("TODO: CAN ONLY WRITE AT $"));
    }

    let key_ptr = ctx.open_key_writable(&key);

    // TODO: modify this: let key_value = key_ptr.get_value::<Value>(&REDIS_JSON_TYPE)?;
    // for now we only allow setting a nonexisting key
    if key_ptr.key_type() != KeyType::Empty {
        return Err(RedisError::Str("TODO: CANNOT NOT SET PREEXISTING KEY"));
    }
    key_ptr.set_value(&REDIS_JSON_TYPE, jsn)?;
    return REDIS_OK;
}
