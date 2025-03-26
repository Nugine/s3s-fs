dev:
    just fetch
    just fmt
    just lint
    just test

fetch:
    cargo fetch

fmt:
    cargo fmt

lint:
    cargo clippy --workspace --all-targets --all-features

test:
    cargo test --workspace --all-targets --all-features

doc:
    RUSTDOCFLAGS="--cfg docsrs" \
    cargo +nightly doc --workspace --all-features --open --no-deps
