#![macro_use]
use common::{random_key, Ctx};
use test_context::test_context;

mod common;

#[test_context(Ctx)]
#[test]
fn simple(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"{"a":{"b":["c"]}}"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$.a.b[0]")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"["c"]"#.as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn recursive_decent(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"{"x":{"a":1},"y":{"a":2}}"#)
        .execute(&mut con);

    // order of matches is arbitary here
    assert!(vec![
        // upstream
        redis::Value::Data("[1,2]".as_bytes().to_vec()),
        // this library
        redis::Value::Data("[2,1]".as_bytes().to_vec())
    ]
    .contains(
        &redis::cmd("JSON.GET")
            .arg(key)
            .arg("$..a")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
    ));
}

#[test_context(Ctx)]
#[test]
fn no_value_matched_at_path(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg("1")
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$.a")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data("[]".as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn no_path_returns_value_at_root(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg("1")
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data("1".as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn multiple_paths_some_are_bad(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"{"a":1,"b":2}"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$.a")
            .arg("$$")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"{"$.a":[1]}"#.as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn multiple_paths(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"{"a":1,"b":2}"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$.a")
            .arg("$.b")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"{"$.a":[1],"$.b":[2]}"#.as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn bad_args_wrong_arity(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    redis::cmd("JSON.GET")
        .query::<redis::Value>(&mut con)
        .expect_err("json get should have failed");
}

#[test_context(Ctx)]
#[test]
fn bad_args_value_is_not_json(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("SET")
        .arg(key.clone())
        .arg("foo")
        .execute(&mut con);

    redis::cmd("JSON.GET")
        .arg(key)
        .query::<redis::Value>(&mut con)
        .expect_err("json get should have failed");
}

#[test_context(Ctx)]
#[test]
fn bad_path(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$$$")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Nil
    );
}

#[test_context(Ctx)]
#[test]
fn key_does_not_exist(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Nil
    );
}

#[test_context(Ctx)]
#[test]
fn format(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"{"a":{"b":["c","d"]}}"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("INDENT")
            .arg("tt")
            .arg("NEWLINE")
            .arg("nn")
            .arg("SPACE")
            .arg("ss")
            .arg("$")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(
            r#"[nntt{nntttt"a":ss{nntttttt"b":ss[nntttttttt"c",nntttttttt"d"nntttttt]nntttt}nntt}nn]"#
                .as_bytes()
                .to_vec()
        )
    );
}

#[test_context(Ctx)]
#[test]
fn format_empty_array(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"[]"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("INDENT")
            .arg("tt")
            .arg("NEWLINE")
            .arg("nn")
            .arg("SPACE")
            .arg("ss")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"[]"#.as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn format_empty_object(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"{}"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("INDENT")
            .arg("tt")
            .arg("NEWLINE")
            .arg("nn")
            .arg("SPACE")
            .arg("ss")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"{}"#.as_bytes().to_vec())
    );
}
