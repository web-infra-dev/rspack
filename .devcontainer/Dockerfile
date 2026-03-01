# Use the [Choice] comment to indicate option arguments that should appear in VS Code UX. Use a comma separated list.
FROM debian:bookworm-slim

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
        ca-certificates \
        curl \
        build-essential \
        pkg-config \
        libssl-dev \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Install rustup with no default toolchain, then install nightly-2025-11-13 with components
# used by rspack (cargo, clippy, rustfmt, rust-src) so agents can run cargo test/check without
# repeated toolchain and crate downloads.
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none \
    && . $CARGO_HOME/env \
    && rustup toolchain install nightly-2025-11-13 \
        -c rustc \
        -c cargo \
        -c rust-std \
        -c clippy \
        -c rustfmt \
        -c rust-src \
    && rustup default nightly-2025-11-13
ENV PATH="${CARGO_HOME}/bin:${PATH}"

# [Optional] Uncomment this section to install additional OS packages.
# RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
#     && apt-get -y install --no-install-recommends <your-package-list-here>
