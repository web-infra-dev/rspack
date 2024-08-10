#!/usr/bin/env -S just --justfile

set windows-shell := ["powershell"]
set shell := ["bash", "-cu"]

_default:
    just --list -u

setup:
  # Rust related setup
  cargo install cargo-binstall
  cargo binstall taplo-cli cargo-release cargo-insta cargo-deny -y
  # Node.js related setup
  corepack enable
  pnpm install

# publish rust crates    
release-crates:
    cargo release publish --no-verify --execute --no-confirm
