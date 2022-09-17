setup() {
    load 'test_helper/bats-support/load'
    load 'test_helper/bats-assert/load'
}

teardown() {
    rm -rf *.rdb
}

@test "save and load" {
    run redis-server --appendonly no --port 9999 --loadmodule $LIB &
    run timeout 10 bash -c 'until printf "" 2>>/dev/null >>/dev/tcp/$0/$1; do sleep 1; done' 0.0.0.0 9999

    run redis-cli -h 0.0.0.0 -p 9999 json.set 1 \$ '"foo"'
    run redis-cli -h 0.0.0.0 -p 9999 save
    run redis-cli -h 0.0.0.0 -p 9999 shutdown

    run redis-server --appendonly no --port 9999 --loadmodule $LIB &
    run timeout 10 bash -c 'until printf "" 2>>/dev/null >>/dev/tcp/$0/$1; do sleep 1; done' 0.0.0.0 9999

    run redis-cli -h 0.0.0.0 -p 9999 json.get 1 \$
    assert_output '["foo"]'

    run redis-cli -h 0.0.0.0 -p 9999 shutdown
}
