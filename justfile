run:
    cargo run \
        --release

debug:
    cargo run \
        --features bevy/dynamic_linking

check:
    cargo +nightly fmt --all -- --check
    cargo check \
        --features bevy/dynamic_linking

test:
    cargo nextest run

fix:
    cargo fix \
        --features bevy/dynamic_linking \
        --all-targets

remote:
    cargo run \
        --release \
        --features bevy/dynamic_linking,remote

fmt:
    cargo +nightly fmt

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
