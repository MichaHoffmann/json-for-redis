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
        0 => match select(jsn, "$")?.pop() {
            Some(v) => Ok(json!(v)),
            None => return Ok(RedisValue::Null),
        },
        1 => match select(jsn, &paths[0].to_string()) {
            Ok(v) => Ok(json!(v)),
            Err(_) => return Ok(RedisValue::Null),
        },
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
        Ok(v) => Ok(RedisValue::SimpleString(to_quoted(&to_string(&v).unwrap()))),
        Err(e) => Err(RedisError::String(e)),
    };
}

// TODO: support for nx or xx logic, jsonpath errors
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
        }
        None => {
            key_ptr.set_value(&REDIS_JSON_TYPE, jsn)?;
        }
    };
    return REDIS_OK;
}

pub fn redis_json_type(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1);

    let key = args.next_arg()?;
    let path = match args.next_string() {
        Ok(v) => v.to_string(),
        Err(_) => "$".to_string(),
    };

    let key_ptr = ctx.open_key_writable(&key);
    let key_value = key_ptr.get_value::<Value>(&REDIS_JSON_TYPE)?;
    let jsn = match key_value {
        Some(v) => v,
        None => return Ok(RedisValue::Null),
    };
    let matches = match select(jsn, &path) {
        Ok(v) => v,
        Err(_) => return Ok(RedisValue::Null),
    };

    if path == "$" {
        let v = unsafe { matches.get_unchecked(0) };
        return Ok(RedisValue::SimpleString(to_quoted(&json_type(v))));
    }
    return Ok(RedisValue::Array(
        matches
            .iter()
            .map(|v| to_quoted(&json_type(v)))
            .map(|v| RedisValue::SimpleString(v))
            .collect(),
    ));
}

fn to_quoted(s: &String) -> String {
    let mut r = s.replace("\"", "\\\"");
    r.insert_str(0, "\"");
    r.insert_str(r.len(), "\"");
    return r;
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
    return r.to_owned();
}
