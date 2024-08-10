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
  
  echo '✅ Setup complete!'

# Check readiness of the project
ready:
  just fmt  
  just check  
  just lint 
  just test
  echo '✅ All passed!'

# Publish Rust crates to crates.io
release:
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

# Run Node.js tests
test-node:
    pnpm install
    pnpm build:cli:debug
    pnpm test:unit