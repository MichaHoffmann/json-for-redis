setup() {
    load '../test_helper/bats-support/load'
    load '../test_helper/bats-assert/load'
}

rcli() {
  redis-cli --no-raw -p $PORT $@
}

@test "json.get - bad args - wrong arity" {
    run rcli json.get
    assert_output "(error) ERR wrong number of arguments for 'json.get' command"
}

@test "json.get - bad args - value not json type" {
    KEY=$(uuid)

    run rcli set $KEY "foo"
    assert_output "OK"

    run rcli json.get $KEY \$
    assert_output "(error) Existing key has wrong Redis type"
}

@test "json.get - bad path" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '"foo"'
    assert_output "OK"

    run rcli json.get x \$...\$\$
    assert_output "(nil)"
}

@test "json.get - key does not exist" {
    KEY=$(uuid)

    run rcli json.get $KEY \$
    assert_output "(nil)"
}

@test "json.get - simple get" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"a":{"b":["c"]}}'
    assert_output "OK"

    run rcli json.get $KEY \$.a.b[0]
    assert_output '"[\"c\"]"'
}

@test "json.get - no path" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '"foo"'
    assert_output "OK"

    run rcli json.get $KEY
    assert_output '"\"foo\""'
}

@test "json.get - value is not matched" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '"a"'
    assert_output "OK"

    run rcli json.get $KEY \$.a.b.c
    assert_output '"[]"'
}

@test "json.get - recursive decent" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"f1":{"a":1},"f2":{"a":2}}'
    assert_output "OK"

    run rcli json.get $KEY \$..a
    assert_output '"[1,2]"'
}

@test "json.get - multiple paths" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"a":1,"b":2}'
    assert_output "OK"

    run rcli json.get $KEY \$.b \$.a
    assert_output --partial '\"$.a\":[1]'
    assert_output --partial '\"$.b\":[2]'
}

@test "json.get - multiple paths, some are bad" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"a":1,"b":2}'
    assert_output "OK"

    run rcli json.get $KEY \$.a \$\$
    assert_output '"{\"$.a\":[1]}"'
}

@test "json.set - bad args - wrong arity - no path" {
    KEY=$(uuid)

    run rcli json.set $KEY '"a"'
    assert_output "(error) ERR wrong number of arguments for 'json.set' command"
}

@test "json.set - bad args - wrong arity - no value" {
    KEY=$(uuid)

    run rcli json.set $KEY \$
    assert_output "(error) ERR wrong number of arguments for 'json.set' command"
}

@test "json.set - bad args - existing key no json" {
    KEY=$(uuid)

    run rcli set $KEY "foo"
    assert_output "OK"

    run rcli json.set $KEY \$ "\"a\""
    assert_output "(error) Existing key has wrong Redis type"
}

@test "json.set - update inner key - simple" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"a":1,"b":2}'
    assert_output "OK"

    run rcli json.set $KEY \$.a '[]'
    assert_output "OK"

    run rcli json.get $KEY \$
    assert_output '"[{\"a\":[],\"b\":2}]"'
}

@test "json.set - update inner key - recursive decent" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"a":{"c":0},"b":{"c":1}}'
    assert_output "OK"

    run rcli json.set $KEY \$..c '2'
    assert_output "OK"

    run rcli json.get $KEY \$
    assert_output '"[{\"a\":{\"c\":2},\"b\":{\"c\":2}}]"'
}

@test "json.type - string" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '"foo"'
    assert_output "OK"

    run rcli json.type $KEY
    assert_output '"string"'
}

@test "json.type - integer" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '1'
    assert_output "OK"

    run rcli json.type $KEY
    assert_output '"integer"'
}

@test "json.type - number" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '1e-6'
    assert_output "OK"

    run rcli json.type $KEY
    assert_output '"number"'
}

@test "json.type - array" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '[]'
    assert_output "OK"

    run rcli json.type $KEY
    assert_output '"array"'
}

@test "json.type - object" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{}'
    assert_output "OK"

    run rcli json.type $KEY
    assert_output '"object"'
}

@test "json.type - boolean" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ 'true'
    assert_output "OK"

    run rcli json.type $KEY
    assert_output '"boolean"'
}

@test "json.type - null" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ 'null'
    assert_output "OK"

    run rcli json.type $KEY
    assert_output '"null"'
}

@test "json.type - negative integer" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '-1'
    assert_output "OK"

    run rcli json.type $KEY
    assert_output '"integer"'
}

@test "json.type - nested - simple match" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"a":1}'
    assert_output "OK"

    run rcli json.type $KEY \$.a
    assert_output '1) "integer"'
}

@test "json.type - nested - recursive decent" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"a":1,"b":{"a":[]}}'
    assert_output "OK"

    run rcli json.type $KEY \$..a
    assert_output '1) "integer"
2) "array"'
}

@test "json.type - nested - no match" {
    KEY=$(uuid)

    run rcli json.set $KEY \$ '{"a":1,"b":{"a":[]}}'
    assert_output "OK"

    run rcli json.type $KEY \$.c
    assert_output "(empty array)"
}
