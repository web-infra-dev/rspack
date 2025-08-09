# Rspack - GitHub Copilot Instructions

## Project Overview

Rspack is a high-performance JavaScript bundler written in Rust that offers strong compatibility with the webpack ecosystem. It provides lightning-fast build speeds while allowing seamless replacement of webpack in existing projects.

### Key Features
- **Fast Startup**: Rust-based architecture for extremely fast build speeds
- **Lightning HMR**: Built-in incremental compilation with fast Hot Module Replacement
- **Webpack Compatible**: Compatible with webpack plugins and loaders
- **Module Federation**: First-class support for Module Federation
- **Production Optimization**: Built-in tree shaking, minification, and other optimizations
- **Framework Agnostic**: Not bound to any specific frontend framework

## Project Architecture

This is a **monorepo** containing both Rust crates and JavaScript packages:

### Rust Components (`crates/` - 88 crates)
- **Core Engine**: Written in Rust for performance
- **Plugins**: Extensive plugin system (css, javascript, html, etc.)
- **Bindings**: Node.js bindings through NAPI
- **Utilities**: Support crates for paths, collections, macros, etc.

Key crates to understand:
- `rspack_core`: Main bundler logic
- `rspack_binding_api`: Node.js API bindings  
- `node_binding`: NAPI interface
- `rspack_plugin_*`: Various bundler plugins

### JavaScript Components (`packages/` - 179 packages)
- **CLI Tools**: Command-line interface and tooling
- **API Wrappers**: JavaScript APIs wrapping Rust core
- **Test Tools**: Testing utilities and frameworks
- **Create Tools**: Project scaffolding tools

## Development Environment

### Prerequisites
- **Rust**: Uses nightly toolchain (see `rust-toolchain.toml`)
- **Node.js**: See `.nvmrc` for required version
- **pnpm**: Version 10.10.0 (enforced in `package.json`)

### Key Commands

```bash
# Build Rust bindings (development)
pnpm run build:binding:dev

# Build JavaScript packages  
pnpm run build:js

# Full development build
pnpm run build:cli:dev

# Run tests
pnpm run test:unit      # JavaScript unit tests
pnpm run test:e2e       # End-to-end tests
pnpm run test:webpack   # Webpack compatibility tests

# Linting
pnpm run lint:js        # JavaScript/TypeScript linting with Biome
pnpm run lint:rs        # Rust dependency checks
pnpm run lint:type      # Type checking with rslint

# Formatting
pnpm run format:js      # Format JavaScript/TypeScript
pnpm run format:rs      # Format Rust code
pnpm run format:toml    # Format TOML files
```

## Code Structure & Conventions

### Rust Code Patterns
- **Edition**: 2024 (see `Cargo.toml`)
- **Toolchain**: Nightly Rust with rust-src component
- **Workspace**: All crates managed in unified workspace
- **Naming**: `rspack_*` prefix for all crates
- **Error Handling**: Use `anyhow` for error handling
- **Async**: Uses `tokio` runtime and `async-trait`

### JavaScript/TypeScript Patterns
- **Formatting**: Disabled in Biome, uses Prettier
- **Linting**: Biome with custom rule overrides
- **Testing**: Jest for unit tests
- **API Design**: TypeScript with API Extractor for public APIs
- **Build**: Uses the Rust core via NAPI bindings

### File Organization
```
rspack/
├── crates/           # Rust source code (88 crates)
├── packages/         # JavaScript packages (179 packages)
├── tests/            # Integration tests
│   ├── webpack-test/ # Webpack compatibility tests
│   ├── e2e/         # End-to-end tests
│   └── bench/       # Benchmarking
├── examples/         # Example projects
├── scripts/          # Build and utility scripts
├── xtask/           # Rust-based task runner
└── website/         # Documentation site
```

## Development Workflow

### Making Changes
1. **Rust changes**: Modify crates, run `pnpm run build:binding:dev`
2. **JavaScript changes**: Modify packages, run `pnpm run build:js` 
3. **Test changes**: Run relevant test suites
4. **Lint changes**: Run linters before committing

### Testing Strategy
- **Unit Tests**: Jest for JavaScript, built-in for Rust
- **Integration Tests**: `tests/webpack-test/` for webpack compatibility
- **E2E Tests**: `tests/e2e/` for end-to-end scenarios
- **Benchmarks**: Performance testing with codspeed

### Build Targets
- **Development**: Fast builds for iteration (`build:dev`)
- **Debug**: Debug builds with symbols (`build:debug`)
- **Release**: Optimized production builds (`build:release`)
- **WASM**: WebAssembly builds for browsers (`build:wasm`)

## Key Directories Explained

- **`crates/rspack_core/`**: Main bundler implementation
- **`crates/node_binding/`**: Node.js bindings via NAPI
- **`crates/rspack_plugin_*/`**: Bundler plugins (CSS, JS, HTML, etc.)
- **`packages/rspack/`**: Main JavaScript API package
- **`packages/rspack-cli/`**: Command-line interface
- **`packages/create-rspack/`**: Project creation tool
- **`packages/rspack-test-tools/`**: Testing utilities
- **`tests/webpack-test/`**: Webpack compatibility test suite
- **`xtask/`**: Custom Rust-based task runner (follows cargo-xtask pattern)

## Plugin Development

Rspack supports both Rust and JavaScript plugins:

### Rust Plugins
- Implement in `crates/rspack_plugin_*/`
- Use the plugin trait system
- Better performance, harder to develop

### JavaScript Plugins  
- Webpack-compatible plugin API
- Easier to develop and debug
- Slower than Rust plugins

## Performance Considerations

- **Hot Paths**: Core bundling logic is in Rust for performance
- **Memory**: Uses efficient Rust collections and memory management
- **Concurrency**: Leverages Rust's async/await and threading
- **Caching**: Built-in caching mechanisms for incremental builds

## Contributing Guidelines

- Follow existing code patterns and conventions
- Add tests for new features
- Update documentation when changing APIs
- Run all linters and tests before submitting
- Use conventional commit messages
- Keep PRs focused and atomic

## Common Debugging Tips

1. **Rust Compilation Issues**: Check `rust-toolchain.toml` and ensure nightly toolchain
2. **Node Binding Issues**: Rebuild bindings with `pnpm run build:binding:dev`
3. **Test Failures**: Check if they're webpack compatibility issues vs. new bugs
4. **Performance**: Use built-in profiling tools and benchmarks
5. **VS Code**: Configured rust-analyzer settings in `.vscode/settings.json`

## Related Projects (Rstack Ecosystem)

- **Rsbuild**: Build tool built on Rspack
- **Rslib**: Library development tool  
- **Rspress**: Static site generator
- **Rsdoctor**: Build analyzer
- **Rstest**: Testing framework
- **Rslint**: Linter

When working on Rspack, consider impacts on the broader Rstack ecosystem.