run-wasm:  # requires https://github.com/jakobhellermann/wasm-server-runner
    CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-server-runner \
    RUSTFLAGS="--cfg web_sys_unstable_apis" \
    cargo run \
        --profile wasm-release \
        --target wasm32-unknown-unknown