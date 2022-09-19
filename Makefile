ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
TARGET=$(ROOT_DIR)/librejson.so
SRC=$(find . -name "*.rs")

all: $(TARGET)

.PHONY: .clean
clean:
	rm -rf $(TARGET) $(UPSTREAM)

$(TARGET): $(SRC)
	cargo build
	cp target/debug/librejson.so $(TARGET)

UPSTREAM=$(ROOT_DIR)/upstream
UPSTREAM_VERSION=7.0.2-RC2
UPSTREAM_TAR=redis-stack-server-$(UPSTREAM_VERSION).rhel8.x86_64.tar.gz
UPSTREAM_URL=https://packages.redis.io/redis-stack/$(UPSTREAM_TAR)
UPSTREAM_ROOT=$(UPSTREAM)/redis-stack-server-$(UPSTREAM_VERSION)
UPSTREAM_REJSON=$(UPSTREAM_ROOT)/lib/rejson.so

$(UPSTREAM_REJSON):
	mkdir -p $(UPSTREAM)
	wget $(UPSTREAM_URL)
	tar -xvf $(UPSTREAM_TAR) --strip-components=1 -C $(UPSTREAM) && rm $(UPSTREAM_TAR)

.PHONY: test-all
test-all: test-integration-target test-persistence-target

.PHONY: test-integration-compare
test-integration-compare: test-integration-upstream test-integration-target

BATS_TIMEOUT=30
BATS_FLAGS=--jobs $(shell nproc) --no-parallelize-across-files --verbose-run
BATS=test/bats/bin/bats $(BATS_FLAGS)

.PHONY: test-integration-target
test-integration-target: $(TARGET) $(UPSTREAM_REJSON)
	LIB=$(TARGET) $(BATS) test/integration/test.bats

.PHONY: test-integration-upstream
test-integration-upstream: $(TARGET) $(UPSTREAM_REJSON)
	LIB=$(UPSTREAM_REJSON) $(BATS) test/integration/test.bats

.PHONY: test-persistence-target
test-persistence-target: $(TARGET) $(UPSTREAM_REJSON)
	LIB=$(TARGET) $(BATS) test/persistence/test.bats
