---
description: 'Rspack is built using Rust and NAPI-RS, then released as Node.js packages'
---

# Prerequisites

Rspack is built using [Rust](https://rust-lang.org/) and [NAPI-RS](https://napi.rs/), then released as [Node.js](https://nodejs.org/) packages.

## Setup Rust

- Install Rust using [rustup](https://rustup.rs/).
- If you are using VS Code, we recommend installing the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension.

On macOS, install the pinned LLD linker from the repository root before building:

```bash
./scripts/macos-linker.sh --setup
```

This installs a separate Rust toolchain for its linker, which supports newer Xcode SDKs. Rspack still uses the compiler pinned in `rust-toolchain.toml`.

## Setup Node.js

Use the latest Node.js LTS version. For installation instructions, see [Install Node.js](https://nodejs.org/en/download).
