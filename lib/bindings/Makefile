default: check

bindings-python:
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/breez_sdk.udl --no-format --language python -o ffi/python

build:
	cargo build

check: fmt-check clippy-check

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
