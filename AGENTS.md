# Rspack – Agent Handbook

Rspack is a Rust-first, webpack-compatible bundler. The repository is a mixed Rust/TypeScript monorepo that ships native bindings for multiple platforms. This handbook gives coding agents the minimum context needed to reason about architecture, dependencies, and workflows.

## Project Snapshot
- **Core idea**: `crates/rspack_core` implements compilation, chunk graph, and plugin hooks in Rust; JavaScript packages expose the API (`@rspack/core`, `@rspack/cli`) and tooling.
- **Runtime bridge**: Platform-specific Node add-ons are built from `crates/node_binding` + `rspack_binding_*` crates and consumed by `@rspack/binding`, which is bundled into `@rspack/core`.
- **Ecosystem parity**: The plugin set intentionally mirrors webpack concepts (CSS handling, HTML integration, MF, HMR, etc.) so most webpack configs run unchanged.

## Repository Topology
- `crates/` – All Rust crates in the workspace. Cargo workspace members are declared in `Cargo.toml` (`crates/*`, `xtask/benchmark`, `xtask`).
- `packages/` – Public npm packages: `@rspack/core`, `@rspack/cli`, `@rspack/browser`, `@rspack/test-tools`, and `create-rspack`.
- `npm/` – Prebuilt platform-specific wrapper packages (linux-x64-gnu, darwin-arm64, etc.) that contain compiled bindings pulled by `@rspack/binding`.
- `tests/` – `tests/rspack-test` (webpack compatibility corpus), `tests/e2e` (scenario-based suites), `tests/bench` (benchmarks).
- `examples/` – Minimal runnable samples that double as integration smoke tests.
- `xtask/` – Rust automation utilities (benchmark harnesses, release helpers).
- `website/` – Docusaurus-powered documentation; kept in sync with package APIs.

## Rust Workspace (`crates/`)
There are ~80 crates. Organize your reasoning with the layers below (dependencies flow top → bottom unless noted).

| Layer | Crates | Dependency Notes |
| --- | --- | --- |
| **Core runtime** | `rspack_core`, `rspack` | `rspack_core` is the compilation engine; `rspack` assembles core with default plugins & loaders. Depends on utilities (`rspack_util`, `rspack_error`, `rspack_storage`, `rspack_collections`, `rspack_hash`, `rspack_workspace`). |
| **Shared utilities** | `rspack_util`, `rspack_error`, `rspack_storage`, `rspack_paths`, `rspack_resolver`, `rspack_fs`, `rspack_tasks`, `rspack_workspace` | Provide error handling, persistence, path logic, resolver logic, async task orchestration, and filesystem abstraction used by nearly every crate. |
| **Compiler extensions** | `rspack_javascript_compiler`, `rspack_regex`, `rspack_ids`, `rspack_location`, `rspack_hook`, `rspack_futures`, `rspack_cacheable*`, `rspack_collections` | Power AST transforms, hashing, deterministic ids, macro helpers, and caching of compilation artifacts. Heavy SWC usage (`swc_core`, `swc_*` crates). |
| **Plugin family** | `rspack_plugin_*` (asset, css, devtool, dll, entry, externals, hmr, html, javascript, json, lazy_compilation, library, mf, progress, rsdoctor, runtime, split_chunks, sri, wasm, worker, etc.) | Each crate implements `Plugin` traits from `rspack_core`. Most depend on `rspack_util`, `rspack_hook`, `rspack_javascript_compiler`, and sometimes SWC extras or lightningcss. Ensure new plugins register with `rspack` crate if they should be bundled by default. |
| **Loader & parser layer** | `rspack_loader_*`, `rspack_loader_runner`, `rspack_loader_testing`, `rspack_cacheable_*` | Provide SWC/LightningCSS powered loaders and test harnesses. They feed transformed modules into `rspack_core`. Optional features on the `rspack` crate gate inclusion. |
| **Bindings & runtime bridge** | `rspack_binding_api`, `rspack_binding_build`, `rspack_binding_builder*`, `rspack_napi*`, `node_binding`, `rspack_watcher` | Generate napi-compatible APIs, build node modules, and expose watch/wrapper utilities. `node_binding` links against the rest of the workspace and exports napi entry points consumed by `@rspack/binding`. |
| **Tooling & tracing** | `rspack_tracing`, `rspack_tracing_perfetto`, `rspack_tasks`, `rspack_watcher`, `rspack_browser`, `rspack_allocator`, `rspack_browserslist` | Provide diagnostics, perfetto tracing export, custom allocator, browser runner shims, and browserslist data. |
| **SWC plugins** | `swc_plugin_import`, `swc_plugin_ts_collector` | Optional SWC plugins for tree-shaking and TS metadata collection used by loaders/plugins. |
| **Automation** | `xtask`, `xtask/benchmark` | CLI helpers that orchestrate benchmarks, fixture generation, and release automation; depend on workspace crates minimally to keep builds fast. |

When touching a crate:
- Review its `Cargo.toml` to understand workspace feature gates; `rspack` exposes most functionality through features like `loader_swc`.
- Keep dependency direction acyclic. New shared helpers usually belong in `rspack_util` or `rspack_workspace` rather than ad-hoc modules.

## JavaScript / TypeScript Packages (`packages/`)

| Package | Purpose | Key Dependencies |
| --- | --- | --- |
| `@rspack/core` (`packages/rspack`) | Primary JS API that mirrors webpack’s compiler/runtime surface. Loads native bindings from `@rspack/binding`, ships built-in plugins/loaders, and exposes hot module runtime shims under `./hot/*`. | Depends on `@rspack/binding` (napi addon built from Rust), `@rspack/lite-tapable`, and tooling such as `memfs`, `enhanced-resolve`, `watchpack`. |
| `@rspack/cli` (`packages/rspack-cli`) | CLI entry (bin `rspack`). Wraps `@rspack/core` with config loading, dev-server coordination, and analyzer hooks. | Depends on `@rspack/core`, `@rspack/dev-server`, `webpack-bundle-analyzer`. |
| `@rspack/browser` | Experimental browser/WASM build of the compiler for playground scenarios; reuses `@napi-rs/wasm-runtime` plus the same tapable layer. | Shares hot runtime code with `@rspack/core`. |
| `@rspack/test-tools` | Harness used by `tests/rspack-test` and downstream consumers. Provides helpers, snapshot tooling, and wasm setup hooks. | Depends on Babel, Jest utilities, `webpack`, and `@rspack/core`. |
| `create-rspack` | Project scaffolding CLI (bin `create-rspack`). Ships templates under `packages/create-rspack/template-*`. | Wraps `create-rstack` to manage prompts and template copying. |

Additional packaging notes:
- `pnpm-workspace.yaml` includes `packages/*`, `scripts`, `website`, bindings (`crates/node_binding`), and large test fixtures so they can share devDependencies.
- `npm/<platform>/package.json` describe binary distribution channels for prebuilt napi artifacts. They are published alongside `@rspack/binding`.

## Dependency Flow Between Rust & JS
1. Rust crates compile into a napi module via `node_binding` + `rspack_binding_api`.
2. `@rspack/binding` (in `packages/rspack` build outputs and `npm/*`) bundles that addon and exposes low-level methods.
3. `@rspack/core` wraps those bindings with JS-friendly classes (Compiler, Watching, MultiCompiler) and injects runtime helpers.
4. CLI/test tools/builders (`@rspack/cli`, `@rspack/test-tools`, `create-rspack`) orchestrate higher-level workflows via the JS API.

## Development Environment
- **Toolchains**: Rust nightly (see `rust-toolchain.toml`), Node.js ≥ 22, pnpm (locked via `.nvmrc` / `package.json` engines).
- **Setup**: `pnpm run setup` installs JS deps and builds initial bindings.
- **Rebuilding bindings**: `pnpm run build:binding:dev` (native, debug); `pnpm run build:cli:dev` does a full binding + JS CLI build.
- **JS builds**: `pnpm run build:js` (all packages) or run `pnpm --filter <pkg> build`.
- **Formatting / linting**: `pnpm run format:rs`, `pnpm run format:js`, `pnpm run lint:js`, `pnpm run lint:type`.
- **Testing**: `pnpm run test:unit`, `pnpm run test:e2e`, `pnpm run test:webpack`. Individual fixtures live in `tests/e2e/cases/**` and `tests/rspack-test/**`.

## Contribution Guardrails
- Align new features with existing plugin architecture; prefer adding a new `rspack_plugin_*` crate rather than bloating `rspack_core`.
- When touching bindings, update both the Rust crates and the npm wrapper (including any `npm/<platform>` manifests).
- Keep docs (`website/` and package READMEs) in sync, especially when modifying public APIs.
- Always run relevant Rust + JS tests locally before opening a PR; CI expects clean `cargo fmt`, `cargo clippy`, Biome, and TypeScript checks.
- Follow conventional commits and keep PRs scoped; cross-language changes should mention the dependency chain explicitly to help reviewers.

This document should remain model-friendly: prefer deterministic commands, keep dependency direction explicit, and document any new crates/packages you introduce so future agents can reason about them quickly.
