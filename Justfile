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

check:
    cargo check \
        --features bevy/dynamic_linking \
        --all-targets

fmt-check:
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

lint: fmt-check clippy

fix:
    cargo fix \
        --features bevy/dynamic_linking \
        --all-targets

run-remote:
    cargo run \
        --release \
        --features bevy/dynamic_linking,remote

install-cargo-tools:
    cargo install --locked cargo-binstall
    cargo binstall --no-confirm cargo-hack
    cargo binstall --no-confirm cargo-nextest
    cargo binstall --no-confirm cargo-machete
    cargo binstall --no-confirm cargo-audit
    cargo binstall --no-confirm wasm-server-runner

install-debian-deps:
    sudo apt update && sudo apt-get install -y --no-install-recommends \
        g++ \
        pkg-config \
        libx11-dev \
        libasound2-dev \
        libudev-dev \
        libxkbcommon-x11-0 \
        libwayland-dev \
        libxkbcommon-dev

# extra
tracy:
    cargo run \
        --profile profiling \
        --features bevy/trace_tracy_memory

flamelens:
    cargo flamegraph \
        --profile profiling \
        --post-process 'flamelens --echo' \
        --root

run-wasm:  # requires https://github.com/jakobhellermann/wasm-server-runner
    CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-server-runner \
    RUSTFLAGS='--cfg=web_sys_unstable_apis --cfg=getrandom_backend="wasm_js"' \
    cargo run \
        --profile wasm-release \
        --target wasm32-unknown-unknown

xvfb-run := if os() == 'linux' {
  'xvfb-run'
} else {
  ''
}

screenshot-and-exit:
    {{xvfb-run}} cargo run \
        --features bevy/bevy_ci_testing,bevy/dynamic_linking