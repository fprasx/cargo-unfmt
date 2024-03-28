run:
    cargo run --release --manifest-path ~/code/rust/cargo-unfmt/Cargo.toml

test:
    cargo test --release --manifest-path ~/code/rust/cargo-unfmt/Cargo.toml && \
    RUSTFLAGS="--cap-lints allow" cargo +nightly-2023-12-28 check --manifest-path ~/code/rust/cargo-unfmt/test_crates/output/rustfmt/Cargo.toml

quality:
    fd -e rs . test_crates/output/rustfmt -X cat | ./avg-line-width.py
