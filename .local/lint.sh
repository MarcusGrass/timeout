#!/bin/sh
cargo fmt --all && cargo clippy -- --warn clippy::pedantic && cargo clippy --tests --features test -- --warn clippy::pedantic
