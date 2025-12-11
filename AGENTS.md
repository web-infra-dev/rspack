# Rspack - GitHub Copilot Instructions

## Project Overview

Rspack is a high-performance JavaScript bundler written in Rust that offers strong compatibility with the webpack ecosystem. It provides lightning-fast build speeds while allowing seamless replacement of webpack in existing projects.

## Project Architecture

- This is a **monorepo** containing both Rust crates and JavaScript packages.
- See [Project Architecture](website/docs/en/contribute/development/project.md) for detailed structure
- **Main directories:**
  - `crates/`: Rust crates (core engine, plugins, utilities)
  - `packages/`: JavaScript/TypeScript packages (`@rspack/core`, `@rspack/cli`, `create-rspack`, etc.)
  - `tests/`: Test suites (`rspack-test/` for integration tests, `e2e/` for E2E tests, `bench/` for benchmarks)
  - `website/`: Documentation site
  - `.github/`: GitHub workflows, issue templates, PR templates

## Setup

- See [Prerequisites](website/docs/en/contribute/development/prerequisites.md) to install Rust toolchain and Node.js
- **Required versions:**
  - Rust: Latest stable (install via [rustup](https://rustup.rs/))
  - Node.js: Latest LTS version
  - pnpm: specified in `package.json`
- Run `pnpm run setup` to install dependencies and build the project

## Building

- **JavaScript packages:** `pnpm run build:js` - Builds all JavaScript/TypeScript packages
- **Rust bindings (dev):** `pnpm run build:binding:dev` - Builds Rust crates in development mode
- **Full build (dev):** `pnpm run build:cli:dev` - Builds both Rust and JavaScript
- **Debug build:** `pnpm run build:binding:debug` - Builds with debug symbols for debugging
- **Release build:** `pnpm run build:binding:release` - Builds optimized release version
- **WASM build:** `pnpm run build:cli:dev:wasm` - Builds WebAssembly version
- **Browser build:** `pnpm run build:cli:dev:browser` - Builds browser-compatible version

## Testing

- **Rust unit tests:** `pnpm run test:rs` - Runs all Rust unit test cases
- **JavaScript unit tests:** `pnpm run test:unit` - Runs all JavaScript test cases
- **E2E tests:** `pnpm run test:e2e` - Runs end-to-end test cases
- **Integration tests:** Run `pnpm run test:base` in `tests/rspack-test` directory
- **HMR tests:** Run `pnpm run test:hot` in `tests/rspack-test` directory
- **Update snapshots:** Add `-u` parameter when snapshots need updating (e.g., `npm run test -- -u`)
- **Filter tests:** Use `-t` parameter to filter test cases (e.g., `npm run test -- -t configCases/asset`)
- See [Testing Guide](website/docs/en/contribute/development/testing.mdx) for detailed test types and structure

## Debugging

- See [Debugging Guide](website/docs/en/contribute/development/debugging.mdx) for detailed instructions
- **VS Code debugging:** Configured in `.vscode/launch.json` with `Debug Rspack` and `Attach` options
- **Rust debugging:** Set breakpoints in Rust code and use `Debug Rspack` or `Attach Rust`
- **JavaScript debugging:** Use `--inspect` or `--inspect-brk` flags, then attach with `Attach JavaScript`
- **Deadlock debugging:** Use `Attach Rust` and pause the debugger to inspect deadlock scenarios
- **rust-lldb:** Use `rust-lldb -- node /path/to/rspack build` to debug panic information

## Code Quality

### Linting

- **JavaScript:** `pnpm run lint:js` - Lints JavaScript/TypeScript code with Biome
- **Type checking:** `pnpm run lint:type` - Checks types with Rslint
- **Rust:** `pnpm run lint:rs` - Lints Rust code with cargo check

### Formatting

- **Rust:** `pnpm run format:rs` - Formats Rust code with cargo fmt
- **JavaScript:** `pnpm run format:js` - Formats JavaScript code with prettier
- **TOML:** `pnpm run format:toml` - Formats TOML files with taplo

### Code Style Guidelines

- **Rust:** Follow standard Rust conventions, use `cargo fmt` for formatting
- **JavaScript/TypeScript:** Follow existing patterns, use Biome for linting and Prettier for formatting
- **Naming:** Use snake_case for Rust, camelCase for JavaScript/TypeScript
- **Imports:** Organize imports logically, follow existing import patterns in the file

## Common Tasks

### Adding a New Feature

1. Create a feature branch from `main`
2. Implement the feature in the appropriate crate/package
3. Add tests (unit tests for Rust, integration tests for JavaScript)
4. Update documentation if APIs change
5. Run linters and tests: `pnpm run lint:js && pnpm run lint:rs && pnpm run test:unit && pnpm run test:rs`
6. Format code: `pnpm run format:rs && pnpm run format:js`
7. Create PR following the PR template

### Modifying Rust Code

- Most core logic is in `crates/rspack_core/`
- Plugins are in `crates/rspack_plugin_*/`
- Utilities are in `crates/rspack_*/`
- After modifying Rust code, rebuild bindings: `pnpm run build:binding:dev`
- Test with: `pnpm run test:rs`

### Modifying JavaScript/TypeScript Code

- Core API is in `packages/rspack/src/`
- CLI is in `packages/rspack-cli/src/`
- Test tools are in `packages/rspack-test-tools/src/`
- After modifying, rebuild: `pnpm run build:js`
- Test with: `pnpm run test:unit`

### Adding Tests

- **Rust tests:** Add `#[test]` functions in the same file or `tests/` directory
- **JavaScript tests:** Add test cases in `tests/rspack-test/{type}Cases/` based on test type
- **Test types:** Normal, Config, Hot, Watch, StatsOutput, StatsAPI, Diagnostic, Hash, Compiler, Defaults, Error, Hook, TreeShaking, Builtin
- See [Testing Guide](website/docs/en/contribute/development/testing.mdx) for test type details

## Dependency Management

- **Package manager:** pnpm (use version from package.json)
- **Workspace:** Uses pnpm workspaces for monorepo management
- **Rust dependencies:** Managed in `Cargo.toml` files in each crate
- **JavaScript dependencies:** Managed in `package.json` files
- **Lock files:** `pnpm-lock.yaml` for JavaScript, `Cargo.lock` for Rust
- **Version consistency:** Use `pnpm run check-dependency-version` to check version consistency

## Performance

- **Benchmarks:** Located in `tests/bench/`
- **Run benchmarks:** `pnpm run bench:ci` (requires setup: `pnpm run bench:prepare`)
- **Profiling build:** `pnpm run build:binding:profiling` - Builds with profiling support
- **Performance tracing:** See `crates/rspack_tracing/` for tracing functionality

## Documentation

- **Documentation site:** `website/` directory
- **English docs:** `website/docs/en/`
- **Chinese docs:** `website/docs/zh/`
- **API documentation:** `website/docs/en/api/`
- **Contribution docs:** `website/docs/en/contribute/`
- **Update docs:** When changing APIs, update relevant documentation files
- **API extraction:** Use `pnpm run api-extractor:local` to extract API documentation

## Pull Request Guidelines

- **Template:** Use `.github/PULL_REQUEST_TEMPLATE.md` as template
- **Description:** State the purpose of the PR and summarize core changes (no need to list every file)
- **Title prefix:**
  - `test:` - For test additions/modifications
  - `fix:` - For bug fixes
  - `feat:` - For new features
  - `refactor:` - For refactoring
  - `chore:` - For maintenance tasks
- **Checklist:** Ensure all items in PR template checklist are completed
- **CI:** All CI checks must pass before merge

## Contributing Guidelines

- **Code patterns:** Follow existing code patterns and conventions in the codebase
- **Tests:** Add tests for new features, update tests when modifying behavior
- **Documentation:** Update documentation when changing APIs or adding features
- **Pre-submit:** Run all linters (`pnpm run lint:js && pnpm run lint:rs`) and tests before submitting
- **Commits:** Use conventional commit messages (see PR title prefixes)
- **PR scope:** Keep PRs focused and atomic - one feature/fix per PR
- **Code review:** Address review comments promptly and update PR accordingly

## Finding Code

- **Rust core:** `crates/rspack_core/` - Core compilation engine
- **Plugins:** `crates/rspack_plugin_*/` - Individual plugin implementations
- **JavaScript API:** `packages/rspack/src/` - Main JavaScript/TypeScript API
- **CLI:** `packages/rspack-cli/src/` - Command-line interface
- **Test utilities:** `packages/rspack-test-tools/src/` - Testing framework
- **Examples:** `examples/` - Example projects
- **Tests:** `tests/rspack-test/` - Integration test cases

## Error Handling

- **Rust errors:** Use `rspack_error` crate for error handling and formatting
- **User-friendly messages:** Errors should provide clear, actionable messages
- **Error context:** Include relevant context (file paths, line numbers, etc.) in error messages
- **Error types:** Use appropriate error types from `rspack_error` crate

## Additional Resources

- **Project structure:** [Project Architecture](website/docs/en/contribute/development/project.md)
- **Testing guide:** [Testing](website/docs/en/contribute/development/testing.mdx)
- **Debugging guide:** [Debugging](website/docs/en/contribute/development/debugging.mdx)
- **Prerequisites:** [Prerequisites](website/docs/en/contribute/development/prerequisites.md)
- **API documentation:** [API Docs](website/docs/en/api/index.mdx)
