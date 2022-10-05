use crate::rejson::REDIS_JSON_TYPE;
use jsonpath_lib::select;
use redis_module::{Context, NextArg, RedisError, RedisResult, RedisString, RedisValue};
use serde::ser::Serialize;
use serde_json::ser::{Formatter, Serializer};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io;

pub fn cmd(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();

    let key = args.next_arg()?;

    let mut fmt = Fmt::new();
    while let Some(s) = args.peek() {
        match s.to_string().as_str() {
            "INDENT" => {
                args.next_arg()?;
                fmt._indent = args.next_string()?;
            }
            "NEWLINE" => {
                args.next_arg()?;
                fmt._newline = args.next_string()?;
            }
            "SPACE" => {
                args.next_arg()?;
                fmt._space = args.next_string()?;
            }
            _ => {
                break;
            }
        }
    }

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
        Ok(v) => {
            let mut w = Vec::with_capacity(128);
            let mut ser = Serializer::with_formatter(&mut w, fmt);
            v.serialize(&mut ser)?;
            return Ok(RedisValue::StringBuffer(w));
        }
        Err(e) => Err(RedisError::String(e)),
    };
}

pub struct Fmt {
    _current_indent: usize,
    _has_value: bool,
    _indent: String,
    _newline: String,
    _space: String,
}

impl Fmt {
    fn new() -> Fmt {
        Fmt {
            _current_indent: 0,
            _has_value: false,
            _indent: String::new(),
            _newline: String::new(),
            _space: String::new(),
        }
    }
}

impl Formatter for Fmt {}
/*
    #[inline]
    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn end_array_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn end_object_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }
}
*/
