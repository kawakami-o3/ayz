.PHONY: run check test fmt check-all

run:
	cargo run

check:
	cargo check

test:
	cargo test

fmt:
	cargo fmt

check-all: check test fmt
