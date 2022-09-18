setup() {
    load '../test_helper/bats-support/load'
    load '../test_helper/bats-assert/load'
}

rcli() {
  redis-cli -p $PORT $@
}

@test "json.get - bad args - wrong arity" {
    run rcli json.get
    assert_output "ERR wrong number of arguments for 'json.get' command"
}

@test "json.get - bad args - value not json type" {
    KEY=$(uuid)

    run rcli set $KEY "foo"
    assert_output "OK"

    run rcli json.get $KEY \$
    assert_output "Existing key has wrong Redis type"
}

@test "json.get - bad path" {
    run rcli json.get x \$\$\$
    assert_output ""
}

@test "json.get - key does not exist" {
    KEY=$(uuid)

    run rcli json.get $KEY \$
    assert_output ""
}

@test "json.get - simple get" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"a":{"b":["c"]}}'
    assert_output "OK"

    run rcli json.get $KEY \$.a.b[0]
    assert_output '["c"]'
}

@test "json.get - value is not matched" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '"a"'
    assert_output "OK"

    run rcli json.get $KEY \$.a.b.c
    assert_output '[]'
}

@test "json.get - recursive decent" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"f1":{"a":1},"f2":{"a":2}}'
    assert_output "OK"

    run rcli json.get $KEY \$..a
    assert_output "[1,2]"
}

@test "json.get - multiple paths" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"a":1,"b":2}'
    assert_output "OK"

    run rcli json.get $KEY \$.b \$.a
    assert_output --partial '"$.a":[1]'
    assert_output --partial '"$.b":[2]'
}

@test "json.set - bad args - wrong arity - no path" {
    KEY=$(uuid)

    run rcli json.set $KEY '"a"'
    assert_output "ERR wrong number of arguments for 'json.set' command"
}

@test "json.set - bad args - wrong arity - no value" {
    KEY=$(uuid)

    run rcli json.set $KEY \$
    assert_output "ERR wrong number of arguments for 'json.set' command"
}

@test "json.set - bad args - existing key no json" {
    KEY=$(uuid)

    run rcli set $KEY "foo"
    assert_output "OK"

    run rcli json.set $KEY \$ "\"a\""
    assert_output "Existing key has wrong Redis type"
}

@test "json.set - update inner key - simple" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"a":1,"b":2}'
    assert_output "OK"

    run rcli json.set $KEY \$.a '[]'
    assert_output "OK"

    run rcli json.get $KEY \$
    assert_output '[{"a":[],"b":2}]'
}

@test "json.set - update inner key - recursive decent" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"a":{"c":0},"b":{"c":1}}'
    assert_output "OK"

    run rcli json.set $KEY \$..c '2'
    assert_output "OK"

    run rcli json.get $KEY \$
    assert_output '[{"a":{"c":2},"b":{"c":2}}]'
}
