# Development commands

Run commands from the repository root unless shown otherwise. Command definitions live in [root package scripts](../package.json), [test package scripts](../tests/rspack-test/package.json), and [Cargo aliases](../.cargo/config.toml).

## Setup

Use the Rust toolchain in `rust-toolchain.toml`, the pnpm version in `package.json`, and the latest Node.js LTS.

`pnpm run setup` installs dependencies and builds the development binding and JS packages.

## Build

Build the changed code before running tests that consume it:

| Changed code | Build                        |
| ------------ | ---------------------------- |
| JavaScript   | `pnpm run build:js`          |
| Rust         | `pnpm run build:binding:dev` |
| Both         | `pnpm run build:cli:dev`     |

Other build variants:

- Native binding variants: `pnpm run build:binding:debug` or `pnpm run build:binding:release`.
- WASM: `pnpm run build:cli:dev:wasm`; browser: `pnpm run build:cli:dev:browser`.

## Tests

- Focused integration cases: `pnpm --dir tests/rspack-test run test -t "configCases/asset"`.
- Base integration suite: `pnpm --dir tests/rspack-test run test:base` (there is no root `test:base` script).
- JavaScript suites: `pnpm run test:unit`; Rust suites: `pnpm run test:rs`.
- HMR: `pnpm run test:hot`; E2E: `pnpm run test:e2e`; API types: `pnpm run test:type`.

See the [testing guide](../website/docs/en/contribute/development/testing.mdx) for harness details. New tests must follow the [repository's test restrictions](../AGENTS.md#adding-tests).

## Lint and formatting

- JavaScript lint: `pnpm run lint:js`.
- Rust check: `pnpm run lint:rs`; clippy: `cargo lint`.
- Rust format check: `cargo fmt --all --check`.
- Format Rust: `pnpm run format:rs`; JS/TS: `pnpm run format:js`.

The root lint and formatting commands cover the whole workspace.

## Performance and debugging

Rust benchmarks live in `xtask/benchmark/`. Build them with `pnpm run build:bench`, then run `pnpm run bench:ci` to prepare fixtures and execute benchmarks. `pnpm run bench:prepare` prepares fixtures separately.

Use `pnpm run build:binding:profiling` for a profiling binding. Tracing support lives in `crates/rspack_tracing/`.

See the [debugging guide](../website/docs/en/contribute/development/debugging.mdx) for VS Code configurations, JavaScript inspection, and `rust-lldb`. See [project layout](../website/docs/en/contribute/development/project.md) for core, plugin, API, CLI, and binding paths.
