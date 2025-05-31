UNAME := $(shell uname)
FFI_DIR := ffi/
BIN_NAME := libbreez_sdk_spark_bindings
TARGET_DIR := ../../target/
RELEASE_DIR := $(TARGET_DIR)release

ifeq ($(UNAME), Darwin)
	CLANG_PREFIX += AR=$(shell brew --prefix llvm)/bin/llvm-ar CC=$(shell brew --prefix llvm)/bin/clang
	BIN_EXT := dylib
else ifeq ($(UNAME), Linux)
	BIN_EXT := so
endif

BIN_PATH := $(RELEASE_DIR)/$(BIN_NAME).$(BIN_EXT)
INSTALL_PREFIX := CARGO_TARGET_DIR="$(TARGET_DIR)"

default: check

bindings-csharp: build-release install-uniffi-bindgen-cs
	uniffi-bindgen-cs --library $(BIN_PATH) --out-dir $(FFI_DIR)csharp --no-format

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

install-uniffi-bindgen-cs:
	$(INSTALL_PREFIX) cargo install uniffi-bindgen-cs --git https://github.com/NordSecurity/uniffi-bindgen-cs --tag v0.9.1+v0.28.3

install-uniffi-bindgen-go:
	$(INSTALL_PREFIX) cargo install uniffi-bindgen-go --git https://github.com/NordSecurity/uniffi-bindgen-go --tag v0.4.0+v0.28.3

install-uniffi-kotlin-multiplatform-bindings:
	$(INSTALL_PREFIX) cargo install --bin uniffi-bindgen-kotlin-multiplatform --git https://gitlab.com/trixnity/uniffi-kotlin-multiplatform-bindings --rev 844a8aee5f1e6aaa9175cde2b0f119aec8357553

test:
	cargo test

wasm-clippy-check:
	$(CLANG_PREFIX) cargo clippy --all-targets --target=wasm32-unknown-unknown -- -D warnings
