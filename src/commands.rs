use redis_module::{Context, NextArg, RedisError, RedisResult, RedisString, RedisValue, REDIS_OK};

use jsonpath_lib::{replace_with, select};
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
        1 => Ok(json!(select(jsn, &paths[0].to_string()).unwrap())),
        _ => {
            let mut m = HashMap::new();
            for path in paths {
                let pp = path.to_string();
                match select(jsn, &pp) {
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

// TODO: support for nx or xx logic
pub fn redis_json_set(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1);

    let key = args.next_arg()?;
    let path = args.next_string()?;
    let val = args.next_string()?;
    // TODO: let nx_or_xx = args.next_string()?;
    args.done()?;

    let key_ptr = ctx.open_key_writable(&key);
    let key_value = key_ptr.get_value::<Value>(&REDIS_JSON_TYPE)?;
    let jsn = from_str::<Value>(&val)?;

    match key_value {
        Some(v) => {
            let res = replace_with(v.clone(), &path.to_string(), &mut |_| Some(jsn.clone()))?;

            key_ptr.set_value(&REDIS_JSON_TYPE, res)?;
            return REDIS_OK;
        }
        None => {
            key_ptr.set_value(&REDIS_JSON_TYPE, jsn)?;
            return REDIS_OK;
        }
    };
}
