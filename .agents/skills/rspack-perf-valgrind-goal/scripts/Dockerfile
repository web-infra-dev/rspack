FROM rust:bookworm

ARG RUST_TOOLCHAIN=nightly-2026-04-16

RUN apt-get update \
  && apt-get install --yes --no-install-recommends \
    build-essential \
    ca-certificates \
    clang \
    cmake \
    curl \
    git \
    jq \
    libssl-dev \
    pkg-config \
    python3 \
    valgrind \
  && rm -rf /var/lib/apt/lists/*

RUN rustup toolchain install "${RUST_TOOLCHAIN}" \
      --profile minimal \
      --component rust-src \
  && rustup default "${RUST_TOOLCHAIN}"

ENV CARGO_INCREMENTAL=0 \
    RUSTUP_TOOLCHAIN=nightly-2026-04-16
