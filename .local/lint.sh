#!/bin/sh
cargo fmt --all && cargo clippy && cargo clippy --all-targets --features test
