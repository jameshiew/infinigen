FROM mcr.microsoft.com/devcontainers/rust:latest

ENV DEBIAN_FRONTEND=noninteractive

RUN apt update && apt-get install -y \
    g++ \
    pkg-config \
    libx11-dev \
    libasound2-dev \
    libudev-dev \
    libxkbcommon-x11-0 \
    libwayland-dev \
    libxkbcommon-dev

RUN rustup target add wasm32-unknown-unknown

RUN cargo install --locked cargo-binstall
RUN cargo binstall --no-confirm just

RUN cargo binstall --no-confirm cargo-hack
RUN cargo binstall --no-confirm cargo-nextest
RUN cargo binstall --no-confirm cargo-machete
RUN cargo binstall --no-confirm cargo-audit