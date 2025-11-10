# Rspack - GitHub Copilot Instructions

## Project Overview

Rspack is a high-performance JavaScript bundler written in Rust that offers strong compatibility with the webpack ecosystem. It provides lightning-fast build speeds while allowing seamless replacement of webpack in existing projects.

## Project Architecture

This is a **monorepo** containing both Rust crates and JavaScript packages:

### Rust crates (`crates/`)

- **Core Engine**: Written in Rust for performance
- **Plugins**: Extensive plugin system (css, javascript, html, etc.)
- **Bindings**: Node.js bindings through NAPI
- **Utilities**: Support crates for paths, collections, macros, etc.

### NPM packages (`packages/`)

- **CLI Tools**: Command-line interface and tooling
- **API Wrappers**: JavaScript APIs wrapping Rust core
- **Test Tools**: Testing utilities and frameworks
- **Create Tools**: Project scaffolding tools

## Development Environment

### Prerequisites

- **Rust**: Uses nightly toolchain (see `rust-toolchain.toml`)
- **Node.js**: 22+
- **pnpm**: enforced in `package.json

### Key Commands

```bash
# Setup up the develop environment
pnpm run setup

# Build Rust bindings (development)
pnpm run build:binding:dev

# Build JavaScript packages
pnpm run build:js

# Full development build
pnpm run build:cli:dev

# Run tests
pnpm run test:unit      # JavaScript unit tests
pnpm run test:e2e       # E2E tests
pnpm run test:webpack   # webpack compatibility tests

# Linting
pnpm run lint:js        # Linting with Biome
pnpm run lint:type      # Type checking with Rslint

# Format
pnpm run format:js      # Format using prettier
pnpm run format:rs      # Format using cargo fmt
```

### File Organization

```
rspack/
├── crates/           # Rust source code
├── packages/         # JavaScript packages
├── tests/            # Integration tests
│   ├── e2e/         # E2E tests
│   └── bench/       # Benchmarking
└── website/         # Documentation site
```

## Contributing Guidelines

- Follow existing code patterns and conventions
- Add tests for new features
- Update documentation when changing APIs
- Run all linters and tests before submitting
- Use conventional commit messages
- Keep PRs focused and atomic
