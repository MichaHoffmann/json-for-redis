setup_suite() {
    PORT=$(python -c "import socket; s = socket.socket(); s.bind(('', 0));print(s.getsockname()[1]);s.close()")
    export PORT

    echo "...starting redis"
    redis-server --appendonly no --save "" --port $PORT --loadmodule $LIB &
    echo "...waiting for redis to start"
    timeout 10 bash -c 'until printf "" 2>>/dev/null >>/dev/tcp/$0/$1; do sleep 1; done' 0.0.0.0 $PORT
    echo "...redis started"
}

teardown_suite() {
    echo "...stopping redis"
    kill $(pidof redis-server)
}
