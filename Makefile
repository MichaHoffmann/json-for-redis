ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
TARGET=$(ROOT_DIR)/librejson.so
SRC=$(find . -name "*.rs")

all: $(TARGET)

.PHONY: .clean
clean:
	rm -rf $(TARGET)

$(TARGET): $(SRC) Cargo.toml Cargo.lock 
	cargo build
	cp target/debug/librejson.so $(TARGET)

.PHONY: test-all
test-all: test-integration test-persistence

.PHONY: test-compare
test-compare: test-integration-real-redis test-integration

BATS_TIMEOUT=30
BATS_FLAGS=--jobs $(shell nproc) --no-parallelize-across-files --verbose-run
BATS=test/bats/bin/bats $(BATS_FLAGS)

.PHONY: test-integration
test-integration: $(TARGET)
	LIB=$(TARGET) $(BATS) test/integration/test.bats

.PHONY: test-integration-real-redis
test-integration-real-redis: $(TARGET)
	BATS_USE_REAL_REDIS=1 $(BATS) test/integration/test.bats

.PHONY: test-persistence
test-persistence: $(TARGET)
	LIB=$(TARGET) $(BATS) test/persistence/test.bats
