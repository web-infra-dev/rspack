# Rspack

## Project Overview

Rspack is a high-performance JavaScript bundler written in Rust that offers strong compatibility with the webpack ecosystem.

## Project Architecture

- **Monorepo** with Rust crates (`crates/`) and JavaScript packages (`packages/`)
- See [Project Architecture](website/docs/en/contribute/development/project.md) for details

## Setup

- **Rust**: Latest stable (via [rustup](https://rustup.rs/))
- **Node.js**: Latest LTS
- **pnpm**: Version in `package.json`
- Run `pnpm run setup` to install and build

## Building

- `pnpm run build:js` - Build JavaScript/TypeScript packages
- `pnpm run build:binding:dev` - Build Rust crates (dev)
- `pnpm run build:cli:dev` - Full build (dev)
- `pnpm run build:binding:debug` - Debug build
- `pnpm run build:binding:release` - Release build
- `pnpm run build:cli:dev:wasm` - WASM build
- `pnpm run build:cli:dev:browser` - Browser build

## Testing

- `pnpm run test:rs` - Rust unit tests
- `pnpm run test:unit` - JavaScript unit tests
- `pnpm run test:e2e` - E2E tests
- `pnpm run test:base` - Integration tests (in `tests/rspack-test`)
- `pnpm run test:hot` - HMR tests
- `cd tests/rspack-test && pnpm run test -t "configCases/asset"` - Run filtered tests

Depends on what you have modified, you need to rebuild by `pnpm run build:js` or `pnpm run build:binding:dev` or `pnpm run build:cli:dev` first, then run testing commands to verify the modification.

## Debugging

- **VS Code**: `.vscode/launch.json` with `Debug Rspack` and `Attach` options
- **Rust**: Set breakpoints, use `Debug Rspack` or `Attach Rust`
- **JavaScript**: Use `--inspect` flag, attach with `Attach JavaScript`
- **rust-lldb**: `rust-lldb -- node /path/to/rspack build` for panic debugging

## Code Quality

- **Linting**: `pnpm run lint:js` (Biome), `pnpm run lint:rs` (cargo check), `pnpm run lint:type` (Rslint)
- **Formatting**: `pnpm run format:rs` (cargo fmt), `pnpm run format:js` (prettier), `pnpm run format:toml` (taplo)
- **Style**: snake_case for Rust, camelCase for JS/TS

## Common Tasks

### Adding a New Feature

1. Create feature branch from `main`
2. Implement in appropriate crate/package
3. Add tests (Rust unit tests, JS integration tests)
4. Update docs if APIs change
5. Run linters and tests: `pnpm run lint:js && pnpm run lint:rs && pnpm run test:unit && pnpm run test:rs`
6. Format: `pnpm run format:rs && pnpm run format:js`
7. Create PR

### Modifying Code

- **Rust**: Core in `crates/rspack_core/`, plugins in `crates/rspack_plugin_*/`, rebuild with `pnpm run build:binding:dev`, test with `pnpm run test:rs`
- **JS/TS**: API in `packages/rspack/src/`, CLI in `packages/rspack-cli/src/`, rebuild with `pnpm run build:js`, test with `pnpm run test:unit`

### Adding Tests

- **Rust**: Add `#[test]` functions in same file or `tests/` directory
- **JavaScript**: Add cases in `tests/rspack-test/{type}Cases/` (Normal, Config, Hot, Watch, StatsOutput, StatsAPI, Diagnostic, Hash, Compiler, Defaults, Error, Hook, TreeShaking, Builtin)

## Dependency Management

- **Package manager**: pnpm (workspaces for monorepo)
- **Rust**: `Cargo.toml` in each crate
- **JavaScript**: `package.json` files
- **Version check**: `pnpm run check-dependency-version`

## Performance

- **Benchmarks**: `tests/bench/`, run with `pnpm run bench:ci` (setup: `pnpm run bench:prepare`)
- **Profiling**: `pnpm run build:binding:profiling`
- **Tracing**: See `crates/rspack_tracing/`

## Documentation

- **Site**: `website/` directory
- **Docs**: `website/docs/en/` (English), `website/docs/zh/` (Chinese)
- **API**: `website/docs/en/api/`
- **Extract API**: `pnpm run api-extractor:local`

## Pull Request Guidelines

- **Template**: Use `.github/PULL_REQUEST_TEMPLATE.md`
- **Title prefix**: `test:`, `fix:`, `feat:`, `refactor:`, `chore:`
- **CI**: All checks must pass

## Contributing

- Follow existing code patterns
- Add tests for new features
- Update docs when APIs change
- Run linters before submitting
- Use conventional commits
- Keep PRs focused (one feature/fix per PR)

## Finding Code

- **Rust core**: `crates/rspack_core/`
- **Plugins**: `crates/rspack_plugin_*/`
- **JavaScript API**: `packages/rspack/src/`
- **CLI**: `packages/rspack-cli/src/`
- **Tests**: `tests/rspack-test/`

## Error Handling

- Use `rspack_error` crate for Rust errors
- Provide clear, actionable error messages
- Include context (file paths, line numbers)

## AI-Friendly Documentation

This project includes comprehensive documentation designed for AI assistants and large language models. All AI-friendly documentation is located in the `agents/` directory:

- **[Architecture Guide](agents/ARCHITECTURE.md)** - High-level architecture overview, core components, compilation pipeline, and system design
- **[API Design](agents/API_DESIGN.md)** - API design principles, patterns, versioning strategy, and compatibility guidelines
- **[Code Style](agents/CODE_STYLE.md)** - Coding standards and conventions for Rust and TypeScript/JavaScript
- **[Common Patterns](agents/COMMON_PATTERNS.md)** - Common code patterns, templates, and best practices for plugin/loader development
- **[Glossary](agents/GLOSSARY.md)** - Comprehensive glossary of terms and concepts used throughout the codebase
- **[Skills](agents/SKILLS.md)** - Required skills and knowledge areas for contributing to Rspack

These documents provide detailed context about the project structure, coding standards, common patterns, and domain-specific knowledge to help AI assistants better understand and contribute to the codebase.

## Resources

- [Project Architecture](website/docs/en/contribute/development/project.md)
- [Testing Guide](website/docs/en/contribute/development/testing.mdx)
- [Debugging Guide](website/docs/en/contribute/development/debugging.mdx)
- [API Docs](website/docs/en/api/index.mdx)
