setup() {
    load '../test_helper/bats-support/load'
    load '../test_helper/bats-assert/load'
}

rcli () {
  redis-cli $@
}

@test "save and load" {
    run rcli json.set 1 \$ '"foo"'
    run rcli save
    run rcli shutdown

    echo "restarting redis to load rdb"
    run redis-server --loadmodule $LIB &
    run timeout 10 bash -c 'until printf "" 2>>/dev/null >>/dev/tcp/$0/$1; do sleep 1; done' 0.0.0.0 6379

    run rcli json.get 1 \$
    assert_output '["foo"]'
}
