# json-for-redis

A WIP Clone of [RedisJSON](https://redis.io/docs/stack/json/).

# Development

It is recommended to use `nix` to fulfill all development dependencies. To activate the development environment simply run `nix-shell` in the project root.

# Testing

To run integration tests simply run:

```bash
nix-shell --run "cargo build && REDIS_JSON_MODULE=$(pwd)/target/debug/librejson.so cargo test"
```

When adding new integration tests please verify that they are testing the correct behaviour by running them against the upstream module:

```bash
nix-shell --run "cargo build && REDIS_JSON_MODULE=/path/to/upstream/librejson.so cargo test"
```
