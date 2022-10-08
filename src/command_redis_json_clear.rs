use crate::rejson::*;
use jsonpath_lib::replace_with;
use redis_module::{Context, NextArg, RedisResult, RedisString, RedisValue};
use serde_json::{Number, Value};

pub fn cmd(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1);

    let key = args.next_arg()?;
    let path = match args.next_string() {
        Ok(v) => v,
        Err(_) => "$".to_owned(),
    };
    args.done()?;

    let key_ptr = ctx.open_key_writable(&key);
    let key_value = key_ptr.get_value::<Value>(&REDIS_JSON_TYPE)?;

    let val = match key_value {
        Some(v) => v,
        None => {
            return Ok(RedisValue::Integer(0));
        }
    };

    let mut i = 0;
    if path == "$" {
        if clear(val) {
            i += 1;
        }
        key_ptr.set_value(&REDIS_JSON_TYPE, val.clone())?;
    } else {
        let res = replace_with(val.clone(), path.as_str(), &mut |mut v| {
            if clear(&mut v) {
                i += 1;
            }
            Some(v)
        })?;
        key_ptr.set_value(&REDIS_JSON_TYPE, res)?;
    }

    Ok(RedisValue::Integer(i))
}

fn clear(v: &mut Value) -> bool {
    if let Some(o) = v.as_object_mut() {
        if o.is_empty() {
            return false;
        }
        o.clear();
        return true;
    }
    if let Some(a) = v.as_array_mut() {
        if a.is_empty() {
            return false;
        }
        a.clear();
        return true;
    }
    if v.is_number() {
        if v.as_f64().unwrap() == 0.0 {
            return false;
        }
        *v = Value::Number(Number::from(0));
        return true;
    }
    false
}
