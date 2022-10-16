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
        .arg(r#"{"obj":{"a":1, "b":2}, "arr":[1,2,3], "str": "foo", "bool": true, "int": 42, "float": 3.14}"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.CLEAR")
            .arg(key.clone())
            .arg("$.*")
            .query::<redis::Value>(&mut con)
            .expect("json clear failed"),
        redis::Value::Int(4)
    );

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(
            r#"[{"obj":{},"arr":[],"str":"foo","bool":true,"int":0,"float":0}]"#
                .as_bytes()
                .to_vec()
        )
    );
}

#[test_context(Ctx)]
#[test]
fn clear_numbers(ctx: &mut Ctx) {
    let mut con = ctx.connection();

    let key = random_key(16);

    redis::cmd("JSON.SET")
        .arg(key.clone())
        .arg("$")
        .arg(r#"{"a":1,"b":0,"c":1.0,"d":0.0,"e":-0,"f":1e63}"#)
        .execute(&mut con);

    assert_eq!(
        redis::cmd("JSON.CLEAR")
            .arg(key.clone())
            .arg("$.*")
            .query::<redis::Value>(&mut con)
            .expect("json clear failed"),
        redis::Value::Int(3)
    );

    assert_eq!(
        redis::cmd("JSON.GET")
            .arg(key)
            .arg("$.*")
            .query::<redis::Value>(&mut con)
            .expect("json get failed"),
        redis::Value::Data(r#"[0,0,0,0.0,-0.0,0]"#.as_bytes().to_vec())
    );
}
