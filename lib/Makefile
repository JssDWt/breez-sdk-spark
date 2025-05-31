SUBDIRS := $(wildcard */.)

.PHONY: $(SUBDIRS)

$(SUBDIRS):
	$(MAKE) -C $@ $(MAKECMDGOALS)

default: check

bindings-csharp: 
	$(MAKE) -C bindings $(MAKECMDGOALS)

bindings-go: 
	$(MAKE) -C bindings $(MAKECMDGOALS)

bindings-kotlin-multiplatform:
	$(MAKE) -C bindings $(MAKECMDGOALS)

bindings-kotlin:
	$(MAKE) -C bindings $(MAKECMDGOALS)

bindings-python:
	$(MAKE) -C bindings $(MAKECMDGOALS)

bindings-swift:
	$(MAKE) -C bindings $(MAKECMDGOALS)

bindings-go: build-release install-uniffi-bindgen-go
	uniffi-bindgen-go --library $(BIN_PATH) --out-dir $(FFI_DIR)go --no-format

bindings-kotlin-multiplatform: build-release install-uniffi-kotlin-multiplatform-bindings
	uniffi-bindgen-kotlin-multiplatform --library $(BIN_PATH) --out-dir $(FFI_DIR)kotlin-multiplatform

bindings-kotlin: build-release
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate --library $(BIN_PATH) --no-format --language kotlin -o $(FFI_DIR)kotlin

bindings-python: build-release
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate --library $(BIN_PATH) --no-format --language python -o $(FFI_DIR)python

bindings-swift: build-release
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate --library $(BIN_PATH) --no-format --language swift -o $(FFI_DIR)swift

build: $(SUBDIRS)

build-release: $(SUBDIRS)

build-wasm: $(SUBDIRS)

check: fmt-check clippy-check test

clippy: clippy-check

clippy-fix: $(SUBDIRS)

clippy-check: $(SUBDIRS)

fix: fmt-fix clippy-fix

fmt: fmt-fix

fmt-fix: $(SUBDIRS)

fmt-check: $(SUBDIRS)

test: $(SUBDIRS)

wasm-clippy-check: $(SUBDIRS)
