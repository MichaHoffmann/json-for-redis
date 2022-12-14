use common::{random_key, Ctx};
use test_context::test_context;

mod common;

#[test_context(Ctx)]
#[test]
fn bad_args_wrong_arity_no_path(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key)
        .arg(r#"{"a":1}"#)
        .query::<redis::Value>(&mut con)
        .expect_err("json set should have failed");
}

#[test_context(Ctx)]
#[test]
fn bad_args_wrong_arity_no_value(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key)
        .arg("$")
        .query::<redis::Value>(&mut con)
        .expect_err("json set should have failed");
}

#[test_context(Ctx)]
#[test]
fn no_json(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("SET")
        .arg(key.clone())
        .arg("foo")
        .execute(&mut con);

    redis::cmd("JSON.SET")
        .arg(key)
        .arg("$")
        .arg(r#""a""#)
        .query::<redis::Value>(&mut con)
        .expect_err("json set should have failed");
}

#[test_context(Ctx)]
#[test]
fn set_at_root(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#""a""#)
        .execute(&mut con);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#""b""#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#""b""#.as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn set_inner_key(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"{"a":1,"b":2}"#)
        .execute(&mut con);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$.a")
        .arg(r#"[]"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"[{"a":[],"b":2}]"#.as_bytes().to_vec())
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
        .arg(r#"{"a":{"c":0},"b":{"c":1}}"#)
        .execute(&mut con);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$..c")
        .arg(r#"2"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"[{"a":{"c":2},"b":{"c":2}}]"#.as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn adding_new_key_to_object(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"{"a":2}"#)
        .execute(&mut con);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$.b")
        .arg(r#"8"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"[{"a":2,"b":8}]"#.as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn adding_new_key_to_object_recursive_decent(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"{"a":{"b":1},"d":{"b":1}}"#)
        .execute(&mut con);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$..b")
        .arg(r#"2"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$..b")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"[2,2]"#.as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn adding_new_key_to_object_deep_recursive_decent(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"{"a":{"b":{"c":1}},"d":{"b":{"c":2}}}"#)
        .execute(&mut con);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$..b.d")
        .arg(r#"3"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$..b.d")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"[3,3]"#.as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn nx_key_does_not_exist(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key)
        .arg("$")
        .arg(r#"{"a":2}"#)
        .arg("NX")
        .execute(&mut con);
}

#[test_context(Ctx)]
#[test]
fn nx_key_does_exist(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#""foo""#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.SET")
            .arg(key)
            .arg("$")
            .arg(r#""bar""#)
            .arg("NX")
            .query::<redis::Value>(&mut con)
            .expect("json set failed"),
        redis::Value::Nil
    );
}

#[test_context(Ctx)]
#[test]
fn xx_key_does_exist(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#""foo""#)
        .execute(&mut con);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#""bar""#)
        .arg("XX")
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"["bar"]"#.as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn xx_key_does_not_exist(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.SET")
            .arg(key)
            .arg("$")
            .arg(r#""bar""#)
            .arg("XX")
            .query::<redis::Value>(&mut con)
            .expect("json set failed"),
        redis::Value::Nil
    );
}
