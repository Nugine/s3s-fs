dev:
    just fetch
    just fmt
    just lint
    just test

fetch:
    cargo fetch

fmt:
    cargo fmt

lint *ARGS:
    cargo clippy --workspace --all-targets {{ARGS}}
    cargo clippy --workspace --all-targets --all-features {{ARGS}}

test:
    cargo test --workspace --all-targets
    cargo test --workspace --all-targets --all-features

doc:
    RUSTDOCFLAGS="--cfg docsrs" \
    cargo +nightly doc --workspace --all-features --open --no-deps

ci:
    cargo fmt --all --check
    just lint -- -D warnings
    just test

install:
    cargo install --path crates/s3s-fs --locked --all-features
