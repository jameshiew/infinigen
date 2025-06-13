FROM mcr.microsoft.com/devcontainers/rust:latest

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt update && apt-get install -y --no-install-recommends \
    g++ \
    pkg-config \
    libx11-dev \
    libasound2-dev \
    libudev-dev \
    libxkbcommon-x11-0 \
    libwayland-dev \
    # for running in devcontainer - https://github.com/bevyengine/bevy/blob/release-0.16.1/.github/workflows/example-run.yml
    xorg \
    libxkbcommon-dev \
    libxkbcommon-x11-0 \
    xvfb \
    libgl1-mesa-dri \
    libxcb-xfixes0-dev \
    mesa-vulkan-drivers \
    && \
    rm -rf /var/lib/apt/lists/*

USER vscode

RUN rustup target add wasm32-unknown-unknown && \
    rustup toolchain install nightly && \
    rustup component add --toolchain nightly rustfmt

RUN cargo install --locked cargo-binstall && \
    cargo binstall --no-confirm just