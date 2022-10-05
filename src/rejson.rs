use redis_module::native_types::RedisType;
use redis_module::raw::{load_string, save_string, RedisModuleTypeMethods};
use redis_module::redisraw::bindings::RedisModuleIO;

use serde_json::{from_str, to_string, Value};

use core::ffi::c_void;
use std::ptr;

pub const MODULE_TYPE_NAME: &str = "RedisJSON";
pub const REDIS_JSON_TYPE_VERSION: i32 = 0;

pub static REDIS_JSON_TYPE: RedisType = RedisType::new(
    MODULE_TYPE_NAME,
    REDIS_JSON_TYPE_VERSION,
    RedisModuleTypeMethods {
        version: redis_module::TYPE_METHOD_VERSION,
        rdb_save: Some(redis_json_rdb_save),
        rdb_load: Some(redis_json_rdb_load),
        aof_rewrite: None,
        free: Some(redis_json_rdb_free),
        mem_usage: None,
        digest: None,
        aux_load: None,
        aux_save: None,
        aux_save_triggers: 0,
        free_effort: None,
        unlink: None,
        copy: None,
        defrag: None,
    },
);

unsafe extern "C" fn redis_json_rdb_load(rdb: *mut RedisModuleIO, _: i32) -> *mut c_void {
    match load_string(rdb) {
        Err(_) => ptr::null_mut(),
        Ok(v) => match from_str::<Value>(&v.to_string_lossy()) {
            Err(_) => ptr::null_mut(),
            Ok(v) => Box::into_raw(Box::new(v)).cast::<c_void>(),
        },
    }
}

unsafe extern "C" fn redis_json_rdb_save(rdb: *mut RedisModuleIO, v: *mut c_void) {
    match to_string(&v.cast::<Value>().as_ref()) {
        Err(e) => panic!("{}", e),
        Ok(v) => save_string(rdb, &v),
    };
}

unsafe extern "C" fn redis_json_rdb_free(v: *mut c_void) {
    if v.is_null() {
        return;
    }
    Box::from_raw(v.cast::<RedisType>());
}
