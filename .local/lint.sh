#!/bin/sh
cargo fmt --all && cargo clippy && cargo clippy --tests --features test
