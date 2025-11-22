wasm_rustflags := '--cfg=web_sys_unstable_apis --cfg=getrandom_backend="wasm_js"'

run *args:
    cargo run {{args}}

dep-check:
    cargo machete
    cargo deny check
    cargo audit

fmt:
    cargo +nightly fmt

test:
    cargo nextest run

doc:
    RUSTDOCFLAGS="-Dwarnings" cargo doc \
        --document-private-items \
        --no-deps

check:
    cargo hack check \
        --all-targets

fmt-check:
    cargo +nightly fmt --all -- --check

clippy-wasm:
    RUSTFLAGS='{{wasm_rustflags}}' \
    cargo clippy \
        --target wasm32-unknown-unknown

clippy-native:
    cargo hack clippy \
        --all-targets \
        --feature-powerset \
        -- -D warnings

clippy: clippy-native clippy-wasm

lint: fmt-check clippy

fix:
    cargo fix \
        --all-targets

run-remote:
    cargo run \
        --release \
        --features remote

install-cargo-tools-essential:
    cargo install --locked cargo-binstall
    cargo binstall --no-confirm cargo-hack
    cargo binstall --no-confirm cargo-nextest

install-cargo-tools: install-cargo-tools-essential
    cargo binstall --no-confirm cargo-machete
    cargo binstall --no-confirm cargo-audit
    cargo binstall --no-confirm cargo-deny
    cargo binstall --no-confirm wasm-server-runner
    cargo install --git https://github.com/TheBevyFlock/bevy_cli --branch main --locked bevy_cli

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
    RUSTFLAGS='{{wasm_rustflags}}' \
    cargo run \
        --features bevy/webgpu \
        --profile wasm-release \
        --target wasm32-unknown-unknown

xvfb-run := if os() == 'linux' {
  'xvfb-run'
} else {
  ''
}

screenshot-and-exit *args:
    {{xvfb-run}} cargo run \
        --features bevy/bevy_ci_testing,bevy/png \
        {{args}}
