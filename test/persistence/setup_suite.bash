setup_suite() {
    echo "...starting redis"

    redis-server --appendonly no --save "" --loadmodule $LIB &

    echo "...waiting for redis to start"
    timeout 10 bash -c 'until printf "" 2>>/dev/null >>/dev/tcp/$0/$1; do sleep 1; done' 0.0.0.0 6379
    echo "...redis started"
}

teardown_suite() {
    echo "...stopping redis"
    kill $(pidof redis-server)
}
