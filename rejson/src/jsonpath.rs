use redis_module::RedisError;
use serde_json::Value;

pub use jsonpath::MapAction;

pub fn get<'a>(path: &str, val: &'a Value) -> Result<Vec<&'a Value>, RedisError> {
    match jsonpath::get(path, val) {
        Ok(v) => Ok(v),
        Err(e) => Err(RedisError::String(e)),
    }
}

pub fn set(path: &str, val: &Value, to: &Value) -> Result<Value, RedisError> {
    match jsonpath::set(path, val, to) {
        Ok(v) => Ok(v),
        Err(e) => Err(RedisError::String(e)),
    }
}

pub fn map_each(
    path: &str,
    val: &Value,
    fun: &mut dyn FnMut(&Value) -> MapAction<Value>,
) -> Result<Value, RedisError> {
    match jsonpath::map_each(path, val, fun) {
        Ok(v) => Ok(v),
        Err(e) => Err(RedisError::String(e)),
    }
}
