export RUSTFLAGS := "-Awarnings"
export RUST_LOG := "info"
export APP_INGESTER_CONFIG_PATH := "./bsky-ingester-app/config"
export APP_OBSERVER_CONFIG_PATH := "./bsky-observer-app/config"

ingester_run:
    cargo run --package bsky-ingester-app --bin bsky_ingester_app

observer_run:
    cargo run --package bsky-observer-app --bin bsky_observer_app

build:
    cargo build

lint:
    cargo fmt --all -- --check

    cargo clippy -- \
        -D warnings \
        -D trivial_casts \
        -D trivial_numeric_casts \
        -D unused_extern_crates \
        -D unused_import_braces \
        -D unused_qualifications \
        -D clippy::all \
        -D clippy::correctness \
        -D clippy::suspicious \
        -D clippy::complexity \
        -D clippy::perf \
        -D clippy::style

clean:
    cargo clean

spin_up:
    docker compose -f docker-compose.yaml up -d

spin_down:
    docker compose -f docker-compose.yaml down
