#!/usr/bin/env -S just --justfile

# Set shell configurations
set windows-shell := ["powershell"]
set shell := ["bash", "-cu"]

# Default target: List all tasks with updated information
_default:
    just --list -u

# Setup environment for Rust and Node.js
setup:
  # Install Rust-related tools
  cargo install cargo-binstall
  cargo binstall taplo-cli cargo-release cargo-insta cargo-deny -y
  
  # Setup Node.js environment
  corepack enable
  pnpm install
  
  @echo '✅ Setup complete!'

# Check readiness of the project
ready:
  just fmt  
  just check  
  just lint 
  just test
  @echo '✅ All passed!'

# Publish Rust crates to crates.io
release-rust:
    cargo release publish --no-verify --execute --no-confirm

# Format Rust, TOML files, and JavaScript code
fmt:
    cargo fmt --all -- --emit=files
    taplo fmt
    pnpm format:js

# Lint Rust and JavaScript code
lint: 
    cargo clippy --workspace --all-targets -- --deny warnings
    pnpm lint:js

# Check Rust code for compilation errors
check:
    cargo check --workspace

# Run tests for both Rust and Node.js
test:
    just test-rust
    just test-node

# Run Rust tests
test-rust:
    cargo test --no-fail-fast

# Supported mode: unit, ci, webpack, plugin
test-node mode="unit":
    pnpm install
    pnpm build:cli:debug
    pnpm test:{{mode}}

# Support `just build [debug|release] (--force)`
build mode="debug" *args="":
    pnpm --filter @rspack/binding build:{{mode}}
    pnpm --filter "@rspack/*" build {{args}}

# Support `just watch [all|rust|node] [debug|release]`
watch target="all" mode="debug":
    just _watch-{{target}} {{mode}}

_watch-all mode:
    pnpm --filter @rspack/binding watch:{{mode}}
    pnpm --filter "@rspack/*" watch

_watch-rust mode:
    pnpm --filter @rspack/binding watch:{{mode}}

_watch-node:
    pnpm --filter "@rspack/*" watch