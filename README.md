# json-for-redis

A WIP Clone of [RedisJSON](https://redis.io/docs/stack/json/).

# Dev Setup

Fedora:
```
dnf -y install rust cargo redis
```

# Tests

Tests use the [BATS](https://bats-core.readthedocs.io/en/stable/) framework. The `tests/integration`
directory contains integration tests. In the current setup they can be run against a redis instance
with the upstream rejson module or a module compiled from this source. The make directive 
`make test-integration-compare` runs the integration test first against a redis that has loaded the
upstream module and then against a redis that has loaded this module.
