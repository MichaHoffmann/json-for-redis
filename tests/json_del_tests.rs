use common::{random_key, Ctx};
use test_context::test_context;

mod common;

#[test_context(Ctx)]
#[test]
fn upstream_example(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"{"a": 1, "b": {"a": 2, "c": 3}}"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.DEL")
            .arg(key.clone())
            .arg("$..a")
            .query::<redis::Value>(&mut con)
            .expect("json del failed"),
        redis::Value::Int(2)
    );

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"[{"b":{"c":3}}]"#.as_bytes().to_vec())
    );
}

#[test_context(Ctx)]
#[test]
fn deleting_root(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"1"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.DEL")
            .arg(key.clone())
            .arg("$")
            .query::<redis::Value>(&mut con)
            .expect("json del failed"),
        redis::Value::Int(1)
    );

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
fn deleting_without_path_deletes_root(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"1"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.DEL")
            .arg(key.clone())
            .query::<redis::Value>(&mut con)
            .expect("json del failed"),
        redis::Value::Int(1)
    );

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Nil
    );
}
