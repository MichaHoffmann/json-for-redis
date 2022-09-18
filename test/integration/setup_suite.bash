setup_suite() {
    echo "...starting redis"

    PORT=$(python -c "import socket; s = socket.socket(); s.bind(('', 0));print(s.getsockname()[1]);s.close()")
    export PORT

    if [[ -n "${BATS_USE_REAL_REDIS}" ]]; then 
      CONTAINER=$(podman run -d -p $PORT:6379 redis/redis-stack:latest)
    else
      redis-server --appendonly no --save "" --port $PORT --loadmodule $LIB &
    fi

    echo "...waiting for redis to start"
    timeout 10 bash -c 'until printf "" 2>>/dev/null >>/dev/tcp/$0/$1; do sleep 1; done' 0.0.0.0 $PORT
    echo "...redis started"
}

teardown_suite() {
    echo "...stopping redis"
    if [[ -n "${BATS_USE_REAL_REDIS}" ]]; then 
      podman stop $CONTAINER
    else
      kill $(pidof redis-server)
    fi

}
