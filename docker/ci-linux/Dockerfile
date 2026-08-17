# syntax=docker/dockerfile:1

FROM ubuntu:22.04

LABEL org.opencontainers.image.source="https://github.com/web-infra-dev/rspack"
LABEL org.opencontainers.image.description="Prebuilt toolchain for rspack Linux CI"
LABEL org.opencontainers.image.licenses="MIT"

SHELL ["/bin/bash", "-euo", "pipefail", "-c"]
ENV DEBIAN_FRONTEND=noninteractive

# clang-15 is required for the powerpc64le / s390x targets: the default clang
# lacks the PowerPC and SystemZ backends. Both are kept so TARGET_CC=clang and
# TARGET_CC=clang-15 keep resolving exactly as they do on GitHub runners.
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential \
      ca-certificates \
      clang \
      clang-15 \
      curl \
      file \
      gcc-riscv64-linux-gnu \
      git \
      libssl-dev \
      pkg-config \
      python3 \
      unzip \
      xz-utils \
      zstd \
    && rm -rf /var/lib/apt/lists/*

# ---------------------------------------------------------------- Node / pnpm
ARG NODE_VERSION=22
ARG COREPACK_VERSION=0.31.0
ARG PNPM_VERSION
RUN : "${PNPM_VERSION:?required build-arg}" \
    && node_full="$(curl -fsSL https://nodejs.org/dist/index.json \
      | python3 -c "import json,sys;print(next(r['version'] for r in json.load(sys.stdin) if r['version'].startswith('v${NODE_VERSION}.')))")" \
    && curl -fsSL "https://nodejs.org/dist/${node_full}/node-${node_full}-linux-x64.tar.xz" \
      | tar -xJ -C /usr/local --strip-components=1 \
        --exclude=CHANGELOG.md --exclude=LICENSE --exclude=README.md \
    && npm install -g "corepack@${COREPACK_VERSION}" --force \
    && corepack enable \
    && corepack prepare "pnpm@${PNPM_VERSION}" --activate \
    && node -v && pnpm -v

# ------------------------------------------------------------------ Rust
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

ARG RUST_TOOLCHAIN
ARG DYLINT_TOOLCHAIN
RUN : "${RUST_TOOLCHAIN:?required build-arg}" "${DYLINT_TOOLCHAIN:?required build-arg}" \
    && curl --proto '=https' --tlsv1.2 --retry 10 --retry-connrefused -fsSL https://sh.rustup.rs \
      | sh -s -- --default-toolchain none -y --no-modify-path \
    && rustup toolchain install "${RUST_TOOLCHAIN}" \
      -c rustc -c cargo -c rust-std -c clippy -c rustfmt -c rust-src \
    && rustup default "${RUST_TOOLCHAIN}" \
    && rustup toolchain install "${DYLINT_TOOLCHAIN}" \
      -c rustc-dev -c llvm-tools-preview -c rust-src

ARG RUST_TARGETS="x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-unknown-linux-musl aarch64-unknown-linux-musl riscv64gc-unknown-linux-gnu riscv64gc-unknown-linux-musl powerpc64le-unknown-linux-gnu s390x-unknown-linux-gnu wasm32-wasip1-threads"
RUN rustup target add --toolchain "${RUST_TOOLCHAIN}" ${RUST_TARGETS}

# --------------------------------------------------------------- Cargo tools
RUN curl -fsSL https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz \
      | tar -xz -C "${CARGO_HOME}/bin"

ARG CARGO_DENY_VERSION=0.18.3
ARG CARGO_SHEAR_VERSION=1.11.2
ARG CARGO_WORKSPACES_VERSION=0.4.0
ARG CARGO_CODSPEED_VERSION=4.4.1
RUN cargo binstall --no-confirm --no-track \
      "cargo-deny@${CARGO_DENY_VERSION}" \
      "cargo-shear@${CARGO_SHEAR_VERSION}" \
      "cargo-workspaces@${CARGO_WORKSPACES_VERSION}" \
      "cargo-codspeed@${CARGO_CODSPEED_VERSION}" \
      cargo-zigbuild \
      taplo-cli

# dylint publishes no prebuilt binaries; this is the compile the image exists to amortize.
RUN cargo install cargo-dylint dylint-link --locked

# ------------------------------------------------------------- Zig / binaryen
ARG ZIG_VERSION=0.14.1
RUN curl -fsSL "https://ziglang.org/download/${ZIG_VERSION}/zig-x86_64-linux-${ZIG_VERSION}.tar.xz" \
      | tar -xJ -C /opt \
    && ln -s "/opt/zig-x86_64-linux-${ZIG_VERSION}/zig" /usr/local/bin/zig \
    && zig version

ARG BINARYEN_VERSION=131
RUN curl -fsSL "https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERSION}/binaryen-version_${BINARYEN_VERSION}-x86_64-linux.tar.gz" \
      | tar -xz -C /opt \
    && ln -s "/opt/binaryen-version_${BINARYEN_VERSION}/bin/wasm-opt" /usr/local/bin/wasm-opt \
    && wasm-opt --version

# The crate registry is rebuilt per-job from the repo's Cargo.lock, so shipping
# it here only inflates the image. chmod last: container jobs may run non-root.
RUN rm -rf "${CARGO_HOME}/registry" "${CARGO_HOME}/git" \
    && chmod -R a+w "${RUSTUP_HOME}" "${CARGO_HOME}"

CMD ["/bin/bash"]
