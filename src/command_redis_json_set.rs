use crate::rejson::*;
use jsonpath_lib::{replace_with, JsonPathError, NodeVisitor, ParseToken, Parser};
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
            match split_remainder_from_path(&path) {
                (p, Some(Remainder::Key(k))) => {
                    let res = replace_with_handle_root(v.clone(), p, &mut |mut vv| {
                        if let Some(o) = vv.as_object_mut() {
                            o.insert(k.to_owned(), jsn.clone());
                        }
                        vv
                    })?;
                    key_ptr.set_value(&REDIS_JSON_TYPE, res)?;
                }
                (p, None) => {
                    let res = replace_with_handle_root(v.clone(), p, &mut |_| jsn.clone())?;
                    key_ptr.set_value(&REDIS_JSON_TYPE, res)?;
                }
            };
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

fn replace_with_handle_root<F>(
    value: Value,
    path: &str,
    fun: &mut F,
) -> Result<Value, JsonPathError>
where
    F: FnMut(Value) -> Value,
{
    if path == "$" {
        return Ok(fun(value));
    }
    replace_with(value, path, &mut |v| Some(fun(v)))
}

#[derive(PartialEq, Eq)]
enum Remainder {
    Key(String),
}

struct NV<'a> {
    input: &'a str,
    stack: Vec<ParseToken>,
}

impl<'a> NV<'a> {
    fn new(input: &'a str) -> Self {
        NV {
            input,
            stack: Vec::new(),
        }
    }

    fn start(&mut self) -> Result<Vec<ParseToken>, String> {
        let node = Parser::compile(self.input)?;
        self.visit(&node);
        Ok(self.stack.split_off(0))
    }
}

impl<'a> NodeVisitor for NV<'a> {
    fn visit_token(&mut self, token: &ParseToken) {
        self.stack.push(token.clone());
    }
}

fn split_remainder_from_path(path: &str) -> (&str, Option<Remainder>) {
    let toks = NV::new(path).start().unwrap();
    match &toks[..] {
        [.., ParseToken::In, ParseToken::Key(k)] => {
            let suffix = format!(".{}", k);
            let pp = path.strip_suffix(&suffix).unwrap();
            (pp, Some(Remainder::Key(k.to_string())))
        }
        _ => (path, None),
    }
}
