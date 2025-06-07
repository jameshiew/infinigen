release:
    cargo run \
        --release

debug:
    cargo run \
        --features bevy/dynamic_linking

machete:
    cargo machete

audit:
    cargo audit

fmt:
    cargo +nightly fmt

test:
    cargo nextest run

doc:
    RUSTDOCFLAGS="-Dwarnings" cargo doc \
        --features bevy/dynamic_linking \
        --document-private-items \
        --no-deps

check-fmt:
    cargo +nightly fmt --all -- --check

clippy-wasm:
    RUSTFLAGS='--cfg=web_sys_unstable_apis --cfg=getrandom_backend="wasm_js"' \
    cargo clippy \
        --target wasm32-unknown-unknown

clippy: clippy-wasm
    cargo hack clippy \
        --all-targets \
        --feature-powerset \
        -- -D warnings

lint: check-fmt clippy

fix:
    cargo fix \
        --features bevy/dynamic_linking \
        --all-targets

run-remote:
    cargo run \
        --release \
        --features bevy/dynamic_linking,remote

run-wasm:  # requires https://github.com/jakobhellermann/wasm-server-runner
    CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-server-runner \
    RUSTFLAGS='--cfg=web_sys_unstable_apis --cfg=getrandom_backend="wasm_js"' \
    cargo run \
        --profile wasm-release \
        --target wasm32-unknown-unknown

tracy:
    cargo run \
        --profile profiling \
        --features bevy/trace_tracy_memory

flamelens:
    cargo flamegraph \
        --profile profiling \
        --post-process 'flamelens --echo' \
        --root
