default: test

build:
    cargo build --workspace --all-targets

test:
    cargo test --workspace --all-targets

fmt:
    cargo fmt --all
    dprint fmt

lint:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -Dwarnings

docs:
    cargo doc --workspace --no-deps --open

calc *ARGS:
    cargo run -p marginalia-calc -- {{ARGS}}

release-dry-run pkg:
    cargo publish -p {{pkg}} --dry-run

clean:
    cargo clean
