use crate::rejson::*;
use jsonpath_lib::{replace_with, JsonPathError, NodeVisitor, ParseToken, Parser};
use redis_module::{Context, NextArg, RedisError, RedisResult, RedisString, RedisValue, REDIS_OK};
use serde_json::{from_str, Value};

// TODO: jsonpath errors, handle adding new values
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

    //TODO: handle '$' path; replace_with does not replace in this case,
    // we can simply overwrite though
    // TODO: handle index append
    match key_value {
        Some(v) => {
            if nx_or_xx.map_or(false, |w| w == Mod::NX) {
                return Ok(RedisValue::Null);
            }
            match split_path(&path) {
                (p, Some(Rem::Key(k))) => {
                    let res = _replace_with(v.clone(), &p, &mut |mut vv| {
                        if let Some(o) = vv.as_object_mut() {
                            o.insert(k.to_owned(), jsn.clone());
                            return Some(vv);
                        }
                        return None;
                    })?;
                    key_ptr.set_value(&REDIS_JSON_TYPE, res)?;
                }
                (p, Some(Rem::Index(_i))) => {
                    let res = _replace_with(v.clone(), &p, &mut |_vv| {
                        return Some(jsn.clone());
                    })?;
                    key_ptr.set_value(&REDIS_JSON_TYPE, res)?;
                }
                (p, None) => {
                    let res = _replace_with(v.clone(), &p, &mut |_| Some(jsn.clone()))?;
                    key_ptr.set_value(&REDIS_JSON_TYPE, res)?;
                }
            };
        }
        None => {
            if nx_or_xx.map_or(false, |w| w == Mod::XX) {
                return Ok(RedisValue::Null);
            }
            key_ptr.set_value(&REDIS_JSON_TYPE, jsn)?;
        }
    };
    return REDIS_OK;
}

#[derive(PartialEq, Eq)]
enum Mod {
    NX,
    XX,
}

#[derive(PartialEq, Eq)]
enum Rem {
    Key(String),
    Index(usize),
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

fn split_path(path: &str) -> (&str, Option<Rem>) {
    let toks = NV::new(path).start().unwrap();
    match &toks[..] {
        [.., ParseToken::In, ParseToken::Key(k)] => {
            let suffix = format!(".{}", k);
            let pp = path.strip_suffix(&suffix.to_string()).unwrap();
            (pp, Some(Rem::Key(k.to_string())))
        }
        _ => return (path, None),
    }
}

fn _replace_with<F>(value: Value, path: &str, fun: &mut F) -> Result<Value, JsonPathError>
where
    F: FnMut(Value) -> Option<Value>,
{
    if path == "$" {
        if let Some(v) = fun(value.clone()) {
            return Ok(v);
        }
        //TODO: This is not correct; should be an error
        return Ok(value);
    }
    return replace_with(value, path, fun);
}
