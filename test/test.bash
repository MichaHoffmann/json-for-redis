setup() {
    load 'test_helper/bats-support/load'
    load 'test_helper/bats-assert/load'
}

@test "json.get - wrong arity" {
    run redis-cli json.get
    assert_output "ERR wrong number of arguments for 'json.get' command"
}

@test "json.get - key does not exist" {
    KEY=$(uuid)

    run redis-cli json.get $KEY \$
    assert_output ""
}

@test "json.get - simple get" {
    KEY=$(uuid)

    run redis-cli json.set $KEY \$ '{"a":{"b":["c"]}}'
    assert_output "OK"

    run redis-cli json.get $KEY \$.a.b[0]
    assert_output '["c"]'
}

@test "json.get - value is not matched" {
    KEY=$(uuid)

    run redis-cli json.set $KEY \$ '"a"'
    assert_output "OK"

    run redis-cli json.get $KEY \$.a.b.c
    assert_output '[]'
}

@test "json.get - recursive decent" {
    KEY=$(uuid)

    run redis-cli json.set $KEY \$ '{"f1":{"a":1},"f2":{"a":2}}'
    assert_output "OK"

    run redis-cli json.get $KEY \$..a
    assert_output "[1,2]"
}

@test "json.get - multiple paths" {
    KEY=$(uuid)

    run redis-cli json.set $KEY \$ '{"a":1,"b":2}'
    assert_output "OK"

    run redis-cli json.get $KEY \$.b \$.a
    assert_output --partial '"$.a":[1]'
    assert_output --partial '"$.b":[2]'
}
