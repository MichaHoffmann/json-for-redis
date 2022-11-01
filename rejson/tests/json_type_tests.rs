use common::{random_key, Ctx};
use test_context::test_context;

mod common;

fn test_type_simple(ctx: &mut Ctx, val: &str, expect: &str) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.SET")
            .arg(key.clone())
            .arg("$")
            .arg(val)
            .query::<redis::Value>(&mut con)
            .expect("json set failed"),
        redis::Value::Okay
    );
    assert_eq!(
        redis::cmd("JSON.TYPE")
            .arg(key)
            .query::<redis::Value>(&mut con)
            .expect("json type failed"),
        redis::Value::Data(expect.as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn string(ctx: &mut Ctx) {
    test_type_simple(ctx, r#""foo""#, "string");
}

#[test_context(Ctx)]
#[test]
fn integer(ctx: &mut Ctx) {
    test_type_simple(ctx, "1", "integer");
}

#[test_context(Ctx)]
#[test]
fn number(ctx: &mut Ctx) {
    test_type_simple(ctx, "1e-6", "number");
}

#[test_context(Ctx)]
#[test]
fn array(ctx: &mut Ctx) {
    test_type_simple(ctx, "[]", "array");
}

#[test_context(Ctx)]
#[test]
fn object(ctx: &mut Ctx) {
    test_type_simple(ctx, "{}", "object");
}

#[test_context(Ctx)]
#[test]
fn boolean(ctx: &mut Ctx) {
    test_type_simple(ctx, "true", "boolean");
}

#[test_context(Ctx)]
#[test]
fn negative_integer(ctx: &mut Ctx) {
    test_type_simple(ctx, "-1", "integer");
}

#[test_context(Ctx)]
#[test]
fn nested_simple_match(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.SET")
            .arg(key.clone())
            .arg("$")
            .arg(r#"{"a":1}"#)
            .query::<redis::Value>(&mut con)
            .expect("json set failed"),
        redis::Value::Okay
    );
    assert_eq!(
        redis::cmd("JSON.TYPE")
            .arg(key)
            .arg("$.a")
            .query::<redis::Value>(&mut con)
            .expect("json type failed"),
        redis::Value::Bulk(vec!(redis::Value::Data("integer".as_bytes().to_vec())))
    );
}

#[test_context(Ctx)]
#[test]
fn nested_recursive_decent(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.SET")
            .arg(key.clone())
            .arg("$")
            .arg(r#"{"a":1,"b":{"a":[]}}"#)
            .query::<redis::Value>(&mut con)
            .expect("json set failed"),
        redis::Value::Okay
    );
    assert_eq!(
        redis::cmd("JSON.TYPE")
            .arg(key)
            .arg("$..a")
            .query::<redis::Value>(&mut con)
            .expect("json type failed"),
        redis::Value::Bulk(vec!(
            redis::Value::Data("integer".as_bytes().to_vec()),
            redis::Value::Data("array".as_bytes().to_vec()),
        ))
    );
}

#[test_context(Ctx)]
#[test]
fn nested_no_match(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    assert_eq!(
        redis::cmd("JSON.SET")
            .arg(key.clone())
            .arg("$")
            .arg(r#"{"a":1,"b":{"a":[]}}"#)
            .query::<redis::Value>(&mut con)
            .expect("json set failed"),
        redis::Value::Okay
    );
    assert_eq!(
        redis::cmd("JSON.TYPE")
            .arg(key)
            .arg("$.c")
            .query::<redis::Value>(&mut con)
            .expect("json type failed"),
        redis::Value::Bulk(vec!())
    );
}
