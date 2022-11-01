use crate::jsonpath::set;
use crate::rejson::*;
use redis_module::{Context, NextArg, RedisError, RedisResult, RedisString, RedisValue, REDIS_OK};
use serde_json::{from_str, Value};

pub fn cmd(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1);

    let key = args.next_arg()?;
    let path = args.next_string()?;
    let val = args.next_string()?;
    let jsn = from_str::<Value>(&val)?;

    let nx_or_xx = match args.next_string() {
        Ok(v) => match v.to_uppercase().as_str() {
            "NX" => Some(Mod::NX),
            "XX" => Some(Mod::XX),
            _ => return Err(RedisError::WrongArity),
        },
        Err(_) => None,
    };
    args.done()?;

    let key_ptr = ctx.open_key_writable(&key);
    let key_value = key_ptr.get_value::<Value>(&REDIS_JSON_TYPE)?;

    match key_value {
        Some(v) => {
            if is_nx(nx_or_xx) {
                return Ok(RedisValue::Null);
            }
            let res = set(path.as_str(), v, &jsn)?;
            key_ptr.set_value(&REDIS_JSON_TYPE, res)?;
        }
        None => {
            if is_xx(nx_or_xx) {
                return Ok(RedisValue::Null);
            }
            key_ptr.set_value(&REDIS_JSON_TYPE, jsn)?;
        }
    };
    REDIS_OK
}

#[derive(PartialEq, Eq)]
enum Mod {
    NX,
    XX,
}

fn is_nx(nx_or_xx: Option<Mod>) -> bool {
    nx_or_xx.map_or(false, |w| w == Mod::NX)
}

fn is_xx(nx_or_xx: Option<Mod>) -> bool {
    nx_or_xx.map_or(false, |w| w == Mod::XX)
}
