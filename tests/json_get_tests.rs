use common::{random_key, Ctx};
use test_context::test_context;

mod common;

#[test_context(Ctx)]
#[test]
fn simple(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.SET")
            .arg(key.clone())
            .arg("$")
            .arg(r#"{"a":{"b":["c"]}}"#)
            .query::<redis::Value>(&mut con)
            .expect("json set failed"),
        redis::Value::Okay
    );
    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key.clone())
            .arg("$.a.b[0]")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Status(r#""[\"c\"]""#.to_string())
    );
}

#[test_context(Ctx)]
#[test]
fn recursive_decent(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.SET")
            .arg(key.clone())
            .arg("$")
            .arg(r#"{"x":{"a":1},"y":{"a":2}}"#)
            .query::<redis::Value>(&mut con)
            .expect("json set failed"),
        redis::Value::Okay
    );
    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key.clone())
            .arg("$..a")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Status(r#""[1,2]""#.to_string())
    );
}

#[test_context(Ctx)]
#[test]
fn no_value_matched_at_path(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.SET")
            .arg(key.clone())
            .arg("$")
            .arg("1")
            .query::<redis::Value>(&mut con)
            .expect("json set failed"),
        redis::Value::Okay
    );
    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key.clone())
            .arg("$.a")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Status(r#""[]""#.to_string())
    );
}

#[test_context(Ctx)]
#[test]
fn no_path_returns_value_at_root(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.SET")
            .arg(key.clone())
            .arg("$")
            .arg("1")
            .query::<redis::Value>(&mut con)
            .expect("json set failed"),
        redis::Value::Okay
    );
    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key.clone())
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Status(r#""1""#.to_string())
    );
}

#[test_context(Ctx)]
#[test]
fn multiple_paths_some_are_bad(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.SET")
            .arg(key.clone())
            .arg("$")
            .arg(r#"{"a":1,"b":2}"#)
            .query::<redis::Value>(&mut con)
            .expect("json set failed"),
        redis::Value::Okay
    );
    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key.clone())
            .arg("$.a")
            .arg("$$")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Status(r#""{\"$.a\":[1]}""#.to_string())
    );
}

#[test_context(Ctx)]
#[test]
fn multiple_paths(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.SET")
            .arg(key.clone())
            .arg("$")
            .arg(r#"{"a":1,"b":2}"#)
            .query::<redis::Value>(&mut con)
            .expect("json set failed"),
        redis::Value::Okay
    );
    let out = redis::cmd("JSON.GET")
        .arg(key.clone())
        .arg("$.a")
        .arg("$.b")
        .query::<redis::Value>(&mut con)
        .expect("json get failed");
    if let redis::Value::Status(res) = out {
        assert!(res.contains(r#"\"$.a\":[1]"#));
        assert!(res.contains(r#"\"$.b\":[2]"#));
    } else {
        panic!("expected redis value");
    };
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
        .arg(key.clone())
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
            .arg(key.clone())
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
            .arg(key.clone())
            .arg("$")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Nil
    );
}