#!/bin/bash
ROOT_DIR="temp/s3s-fs"
mkdir -p "$ROOT_DIR"

if [ -n "$1" ]; then
	ROOT_DIR="$1"
fi

if [ -z "$RUST_LOG" ]; then
    export RUST_LOG="s3s_fs=debug,s3s=debug"
fi

if [ -z "$PROFILE" ]; then
    PROFILE="release"
fi

cargo run -p s3s-fs --all-features --profile "$PROFILE" -- \
    --root          "$ROOT_DIR"     \
    --access-key    AKEXAMPLES3S    \
    --secret-key    SKEXAMPLES3S    \
    --host          localhost       \
    --port          8014            \
    --domains       localhost:8014  \
    --domains       localhost       \
