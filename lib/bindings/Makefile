UNAME := $(shell uname)
FFI_DIR := ffi/
BINNAME := libbreez_sdk_bindings
RELEASE_DIR := ../../target/release

ifeq ($(UNAME), Darwin)
	CLANG_PREFIX += AR=$(shell brew --prefix llvm)/bin/llvm-ar CC=$(shell brew --prefix llvm)/bin/clang
	BINEXT := dylib
else ifeq ($(UNAME), Linux)
	BINEXT := so
endif

BIN_PATH := $(RELEASE_DIR)/$(BINNAME).$(BINEXT)

default: check

bindings-kotlin: build-release
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate --library $(BIN_PATH) --no-format --language kotlin -o $(FFI_DIR)kotlin

bindings-python: build-release
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate --library $(BIN_PATH) --no-format --language python -o $(FFI_DIR)python

bindings-swift: build-release
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate --library $(BIN_PATH) --no-format --language swift -o $(FFI_DIR)swift

build:
	cargo build

build-release:
	cargo build --release

build-wasm:
	$(CLANG_PREFIX) cargo build --target=wasm32-unknown-unknown

check: fmt-check clippy-check wasm-clippy-check

clippy: clippy-check

clippy-fix:
	cargo clippy --fix -- -D warnings
	cargo clippy --fix --tests -- -D warnings

clippy-check:
	cargo clippy -- -D warnings
	cargo clippy --tests -- -D warnings

fix: fmt-fix clippy-fix

fmt: fmt-fix

fmt-fix:
	cargo fmt

fmt-check:
	cargo fmt -- --check

test:
	cargo test

wasm-clippy-check:
	$(CLANG_PREFIX) cargo clippy --all-targets --target=wasm32-unknown-unknown -- -D warnings
