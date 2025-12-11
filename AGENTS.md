# Rspack – Crate Companion

Rspack is a Rust-first, webpack-compatible bundler. The repository mixes Rust crates and TypeScript packages; this document focuses on the Rust side so agents can reason about every crate’s responsibility before touching code.

## Quick Orientation
- `crates/` holds all Rust workspace members listed in the root `Cargo.toml` (`crates/*`, `xtask`, `xtask/benchmark`).
- Node consumers talk to the Rust engine through the napi stack (`node_binding` + `rspack_binding_*`), which is packaged into `@rspack/binding` and imported by `@rspack/core`.
- Default behavior is assembled from many fine-grained plugins; understanding which crate owns a capability prevents duplicating logic.

## Dev Workflow Cheat Sheet
- Toolchains: Rust nightly (`rust-toolchain.toml`), Node ≥ 22, pnpm (see `.nvmrc`).
- Bootstrap once via `pnpm run setup`, then rebuild bindings with `pnpm run build:binding:dev` or `pnpm run build:cli:dev`.
- `pnpm run build:js`, `pnpm --filter <pkg> build`, or `cargo test -p <crate>` keep language-specific feedback loops tight.
- Format & lint with `pnpm run format:rs/js` plus `pnpm run lint:js`, `pnpm run lint:type`.
- Run tests using `pnpm run test:unit`, `pnpm run test:e2e`, `pnpm run test:webpack`, or targeted Rust `cargo test`.

## Crate Catalog

### Core Engine & Workspace
- `rspack_core`: Compilation engine that models modules, chunks, dependency graphs, and plugin hooks.
- `rspack`: Bundles `rspack_core` with default plugins/loaders and exposes feature flags such as `loader_swc`.
- `rspack_workspace`: Workspace-level helpers for config, feature wiring, and shared constants.
- `rspack_storage`: Persistent cache and serialization layer backing incremental builds.
- `rspack_paths`: Cross-platform path normalization and resolver helpers.
- `rspack_fs`: Virtual + native filesystem abstraction with memory FS support.
- `rspack_tasks`: Async orchestration primitives shared across watchers, compilers, and tooling.
- `rspack_browser`: Runtime shims for executing the compiler inside browser/WASM contexts.
- `rspack_browserslist`: Embedded browserslist database plus query helpers used for target resolution.
- `rspack_allocator`: Custom allocator glue (based on `mimalloc`) tuned for bundler workloads.
- `rspack_tracing`: Tracing subscribers and span helpers for diagnostic logging.
- `rspack_tracing_perfetto`: Perfetto exporter that converts tracing data into Perfetto-readable streams.
- `rspack_watcher`: File watching abstraction that powers incremental compilation.

### Utility & Shared Services
- `rspack_util`: General-purpose helpers (hashing, path logic, formatting, env detection).
- `rspack_collections`: Domain-specific data structures optimized for compiler hot paths.
- `rspack_error`: Error types with rich diagnostics and miette integration.
- `rspack_futures`: Async utilities and combinators tailored to the bundler.
- `rspack_hash`: Hash helpers (xxhash, md4, etc.) used for asset and chunk hashing.
- `rspack_ids`: Deterministic ID assignment for modules, chunks, and runtimes.
- `rspack_location`: Source-location bookkeeping reused by diagnostics and loaders.
- `rspack_regex`: Regex helpers and compiled patterns for loaders/resolvers.
- `rspack_hook`: Trait definitions and dispatch utilities for the plugin system.
- `rspack_javascript_compiler`: SWC-powered compilation frontend for JavaScript/TypeScript modules.
- `rspack_cacheable`: Cacheable trait implementations and helpers for memoizing structures.
- `rspack_cacheable_macros`: Procedural macros that derive cacheable implementations.
- `rspack_cacheable_test`: Test suite validating cacheable behavior and stability.

### Binding & NAPI Stack
- `node_binding`: Binary crate that links the Rust engine into a napi module consumed by Node.js.
- `rspack_binding_api`: Type-safe bridge exposing Rust APIs to the napi surface.
- `rspack_binding_build`: Build scripts and shared logic for compiling/publishing bindings.
- `rspack_binding_builder`: Helpers that map Rust structs/enums to napi objects.
- `rspack_binding_builder_macros`: Macros backing the binding builder DSL.
- `rspack_binding_builder_testing`: Regression tests for generated binding code.
- `rspack_napi`: napi runtime glue, wrapping compiler operations for JavaScript callers.
- `rspack_napi_macros`: Attribute macros that simplify napi export declarations.

### Loader, Macro & Testing Crates
- `rspack_loader_runner`: Rust reimplementation of webpack’s loader-runner for executing loader chains.
- `rspack_loader_swc`: Loader that leverages SWC for JS/TS transpilation.
- `rspack_loader_lightningcss`: Loader providing LightningCSS parsing/minification.
- `rspack_loader_react_refresh`: Injects React Refresh runtime hooks during compilation.
- `rspack_loader_preact_refresh`: Equivalent hot-refresh support for Preact.
- `rspack_loader_testing`: Shared fixtures and asserts for loader behavior.
- `rspack_macros`: Shared procedural macros (hooks, visitors, AST utilities).
- `rspack_macros_test`: Macro regression and compile-fail tests ensuring macro stability.

### Plugin Crates

#### General & Entry Logic
- `rspack_plugin_asset`: Enables asset modules and URL/inline emission strategies.
- `rspack_plugin_banner`: Injects banner comments or license headers into assets.
- `rspack_plugin_circular_dependencies`: Detects and reports circular import graphs.
- `rspack_plugin_copy`: Copies static files/folders into the output directory.
- `rspack_plugin_devtool`: Implements devtool/source-map related hooks.
- `rspack_plugin_dll`: Provides webpack-style DLL (precompiled bundle) support.
- `rspack_plugin_dynamic_entry`: Resolves entry points generated at runtime/function form.
- `rspack_plugin_ensure_chunk_conditions`: Validates chunk graph invariants before emit.
- `rspack_plugin_entry`: Normalizes and wires standard entry configurations.
- `rspack_plugin_esm_library`: Adjusts runtime for ESM-targeted library builds.
- `rspack_plugin_externals`: Marks modules as external and skips bundling.
- `rspack_plugin_html`: Generates HTML assets from templates (HtmlWebpackPlugin parity).
- `rspack_plugin_ignore`: Skips specified modules/files entirely.
- `rspack_plugin_javascript`: Default JavaScript module processing pipeline.
- `rspack_plugin_json`: Handles JSON modules, default exports, and tree-shaking.
- `rspack_plugin_library`: Configures library target wrappers (UMD, global, etc.).
- `rspack_plugin_module_info_header`: Prepends module metadata comments.
- `rspack_plugin_module_replacement`: Implements NormalModuleReplacementPlugin behavior.
- `rspack_plugin_no_emit_on_errors`: Cancels asset emit when compilation errors occur.
- `rspack_plugin_progress`: Emits human-readable compilation progress updates.
- `rspack_plugin_rsdoctor`: Integrates rsdoctor diagnostic reporting.
- `rspack_plugin_rslib`: Utilities for bundling the rslib-based toolchain.
- `rspack_plugin_rstest`: Hooks specific to rspack’s regression-testing harness.
- `rspack_plugin_warn_sensitive_module`: Warns when flagged modules enter the graph.

#### CSS & Asset Flow
- `rspack_plugin_css`: Core CSS parser/generator supporting modules and layering.
- `rspack_plugin_css_chunking`: Splits CSS output per chunk to avoid duplication.
- `rspack_plugin_extract_css`: Emits CSS into external files instead of JS strings.
- `rspack_plugin_lightning_css_minimizer`: Minifies CSS via LightningCSS bindings.

#### Chunking & Optimization
- `rspack_plugin_limit_chunk_count`: Caps the number of emitted chunks.
- `rspack_plugin_merge_duplicate_chunks`: Collapses chunks with identical content.
- `rspack_plugin_real_content_hash`: Produces stable, content-derived chunk hashes.
- `rspack_plugin_remove_duplicate_modules`: Deduplicates module instances across chunks.
- `rspack_plugin_remove_empty_chunks`: Drops chunks that no longer contain modules.
- `rspack_plugin_runtime_chunk`: Extracts runtime logic into standalone chunks.
- `rspack_plugin_size_limits`: Emits warnings when assets exceed configured budgets.
- `rspack_plugin_split_chunks`: Implements webpack’s SplitChunksPlugin heuristics.

#### Runtime, Federation & Workers
- `rspack_plugin_hmr`: Hot Module Replacement runtime injection.
- `rspack_plugin_lazy_compilation`: Defers compilation until a module is requested.
- `rspack_plugin_mf`: Module Federation runtime, manifests, and shared scope wiring.
- `rspack_plugin_runtime`: Core runtime template and bootstrap logic.
- `rspack_plugin_schemes`: Supports custom URL schemes (data:, http:, etc.).
- `rspack_plugin_sri`: Generates Subresource Integrity hashes.
- `rspack_plugin_swc_js_minimizer`: JavaScript minimizer built on SWC.
- `rspack_plugin_wasm`: WebAssembly module loading/runtime helpers.
- `rspack_plugin_web_worker_template`: Generates web worker wrapper code.
- `rspack_plugin_worker`: Handles worker entrypoints across targets.

### SWC Plugins
- `swc_plugin_import`: SWC plugin that transforms import patterns (used by plugin_import functionality).
- `swc_plugin_ts_collector`: Collects TypeScript metadata required for advanced analysis.

### Automation & Benchmarks
- `xtask`: Cargo xtask-style CLI for releasing, linting, and repository automation.
- `rspack_benchmark` (`xtask/benchmark`): Criterion-based benchmark harness targeting hot compiler paths.

## Packaging Notes
- `packages/` exposes the JS surface area (`@rspack/core`, `@rspack/cli`, `@rspack/browser`, `@rspack/test-tools`, `create-rspack`).
- `npm/<triple>/` packages publish prebuilt napi binaries per platform/arch; keep them updated when bindings change.
- `pnpm-workspace.yaml` anchors which directories participate in JS dependency hoisting.

## Contribution Guardrails
- Extend existing crates instead of duplicating logic; if behavior affects plugins, update the relevant `rspack_plugin_*` crate.
- Updating the napi stack requires touching both Rust bindings and the JS consumers in `packages/`.
- Document public API changes in `website/` and package READMEs.
- Keep commits conventional and scoped; cross-language changes should mention the dependency chain explicitly.

This reference should remain model-friendly: if you add a crate, include a one-line summary here so future agents can navigate the workspace without spelunking the codebase.
