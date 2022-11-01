use crate::jsonpath::get;
use crate::rejson::REDIS_JSON_TYPE;
use redis_module::{Context, NextArg, RedisError, RedisResult, RedisString, RedisValue};
use serde::ser::Serialize;
use serde_json::ser::{Formatter, Serializer};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io;

pub fn cmd(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    let mut args = args.into_iter().skip(1).peekable();

    let key = args.next_arg()?;

    let mut fmt = CustomFormatter::new();
    while let Some(s) = args.peek() {
        match s.to_string().as_str() {
            "INDENT" => {
                args.next_arg()?;
                fmt.indent = args.next_string()?;
            }
            "NEWLINE" => {
                args.next_arg()?;
                fmt.newline = args.next_string()?;
            }
            "SPACE" => {
                args.next_arg()?;
                fmt.space = args.next_string()?;
            }
            _ => {
                break;
            }
        }
    }
    let paths = args.collect::<Vec<RedisString>>();

    let key_ptr = ctx.open_key_writable(&key);
    let key_value = key_ptr.get_value::<Value>(&REDIS_JSON_TYPE)?;
    let jsn = match key_value {
        Some(v) => v,
        None => return Ok(RedisValue::Null),
    };

    let res = match paths.len() {
        0 => Ok(json!(jsn)),
        1 => match get(&paths[0].to_string(), jsn) {
            Ok(v) => Ok(json!(v)),
            Err(_) => return Ok(RedisValue::Null),
        },
        _ => {
            let m = paths
                .iter()
                .map(|p| p.to_string())
                .map(|p| (p.clone(), get(&p, jsn)))
                .filter(|(_, r)| r.is_ok())
                .map(|(p, r)| (p, r.unwrap()))
                .collect::<HashMap<String, Vec<&Value>>>();
            Ok(json!(m))
        }
    };

    match res {
        Ok(v) => {
            let mut w = Vec::with_capacity(128);
            let mut ser = Serializer::with_formatter(&mut w, fmt);
            v.serialize(&mut ser)?;
            Ok(RedisValue::StringBuffer(w))
        }
        Err(e) => Err(RedisError::String(e)),
    }
}

pub struct CustomFormatter {
    indent: String,
    newline: String,
    space: String,
    current_indent: usize,
    has_value: bool,
}

impl CustomFormatter {
    fn new() -> CustomFormatter {
        CustomFormatter {
            indent: String::new(),
            newline: String::new(),
            space: String::new(),
            current_indent: 0,
            has_value: false,
        }
    }
}

// basically https://docs.rs/serde_json/latest/src/serde_json/ser.rs.html#1957
impl Formatter for CustomFormatter {
    #[inline]
    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent += 1;
        self.has_value = false;
        writer.write_all(b"[")
    }
    #[inline]
    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent -= 1;
        if self.has_value {
            writer.write_all(self.newline.as_bytes())?;
            writer.write_all(self.indent.repeat(self.current_indent).as_bytes())?;
        }
        writer.write_all(b"]")
    }

    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if first {
            writer.write_all(self.newline.as_bytes())?;
        } else {
            writer.write_all(b",")?;
            writer.write_all(self.newline.as_bytes())?;
        }
        writer.write_all(self.indent.repeat(self.current_indent).as_bytes())
    }

    #[inline]
    fn end_array_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.has_value = true;
        Ok(())
    }

    #[inline]
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent += 1;
        self.has_value = false;
        writer.write_all(b"{")
    }

    #[inline]
    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.current_indent -= 1;
        if self.has_value {
            writer.write_all(self.newline.as_bytes())?;
            writer.write_all(self.indent.repeat(self.current_indent).as_bytes())?;
        }
        writer.write_all(b"}")
    }

    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if first {
            writer.write_all(self.newline.as_bytes())?;
        } else {
            writer.write_all(b",")?;
            writer.write_all(self.newline.as_bytes())?;
        }
        writer.write_all(self.indent.repeat(self.current_indent).as_bytes())
    }

    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        writer.write_all(b":")?;
        writer.write_all(self.space.as_bytes())
    }
    #[inline]
    fn end_object_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.has_value = true;
        Ok(())
    }
}
