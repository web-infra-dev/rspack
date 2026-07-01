---
name: rspack-binary-size
description: Use when investigating Rspack binary size, reviewing size growth in a change, or looking for opportunities to reduce macro expansion, generic monomorphization, duplicate dependencies, repeated glue code, trampoline code, shared stubs, and feature-level bloat.
---

# Rspack Binary Size Analysis

Use this skill when the task is about native binary size, Node binding size, Rust codegen bloat, macro expansion, generic expansion, duplicate dependencies, or reviewing a pull request for binary-size regressions.

The goal is to collect evidence first, then decide whether the growth comes from:

- repeated macro-generated glue
- generic monomorphization
- duplicate dependency versions or feature sets
- always-linked builtins, loaders, debug APIs, or optional integration crates
- repeated wrappers that can be replaced by shared helpers, trampolines, stubs, or type-erased adapters

## Collection Scripts

Run these from the repository root.

```sh
pnpm --filter @rspack/skill-binary-size macro-expansion-stats
pnpm --filter @rspack/skill-binary-size generic-expansion-stats
pnpm --filter @rspack/skill-binary-size duplicate-deps
```

Reports are written under `target/binary-size-reports/`.

For pull request review, collect reports on the base revision and the candidate revision, then compare the summaries.

## Macro Expansion Review

Start with `macro-expansion-stats.mjs`. The macro script is intentionally generic. It scans Rust source for:

- attribute-like macro candidates, such as `#[napi]`, `#[plugin_hook]`, `#[cacheable]`, and derive attributes
- function-like macro invocations, such as `define_hook!`, `impl_module_methods!`, and local helper macros
- derive macro names inside `#[derive(...)]`

Use optional expansion backends when counts alone are not enough:

```sh
MACRO_EXPAND_BACKEND=rust-analyzer pnpm --filter @rspack/skill-binary-size macro-expansion-stats
MACRO_EXPAND_BACKEND=cargo-expand CARGO_EXPAND_CRATES='rspack_binding_api rspack_core' pnpm --filter @rspack/skill-binary-size macro-expansion-stats
```

The rust-analyzer backend uses the custom LSP request `rust-analyzer/expandMacro` and records expansion size for candidates that the language server can expand. The cargo-expand backend records crate-level expanded output and marker counts. Prefer rust-analyzer for many individual macro locations and cargo-expand for whole-crate before/after comparisons.

Useful environment variables:

- `MACRO_EXPAND_BACKEND=none|rust-analyzer|cargo-expand`
- `RA_EXPAND_LIMIT=200`
- `RA_EXPAND_FILTER='napi|cacheable|define_hook'`
- `RUST_ANALYZER=rust-analyzer`
- `CARGO_EXPAND_CRATES='rspack_binding_api'`
- `MACRO_SCAN_ROOTS='crates packages'`
- `MACRO_SCAN_MAX_BYTES=1048576`

Treat high counts as a queue for source review, not as proof of size by itself.

Pay special attention to:

- `#[napi]`, `#[napi(object)]`, `#[napi(string_enum)]`, and generated N-API classes or objects
- `#[cacheable]` and `#[cacheable_dyn]`
- `#[plugin_hook]`, `define_hook!`, and hook registration macros
- `#[impl_runtime_module]`
- `#[implemented_javascript_parser_hooks]`
- `derive(RspackHashable)` and `derive(MergeFrom)`
- binding API helper macros such as module wrappers and symbol export helpers

For `#[napi(object)]`, check directionality first. Input-only DTOs should use `object_to_js = false`; output-only DTOs should use `object_from_js = false`. Leaving a DTO as plain `#[napi(object)]` generates both conversion directions.

## Generic Expansion Review

Run `generic-expansion-stats.mjs` after building the relevant target. By default it groups demangled symbols from existing target artifacts, which is fast and does not trigger a release rebuild. Enable `cargo bloat` explicitly when section-level attribution is needed:

```sh
RUN_CARGO_BLOAT=1 pnpm --filter @rspack/skill-binary-size generic-expansion-stats
```

Useful environment variables:

- `RUN_CARGO_BLOAT=1`
- `CARGO_BLOAT_PACKAGE=rspack_node`
- `OUT_DIR=target/binary-size-reports/custom-generic-report`

High-priority generic expansion sources include:

- `ThreadsafeFunction<T, R>` and wrappers around it
- `Function<Args, Return>`
- `Promise<T>`
- `Either`, `Either3`, and larger union-like generic types
- `Vec<T>`, `Option<T>`, `Result<T>`, and nested combinations used at the N-API boundary
- generated `FromNapiValue`, `ToNapiValue`, `ValidateNapiValue`, and `TypeName` implementations
- hook tap structs parameterized by hook argument and return types

Do not replace static dispatch inside parser, module graph, dependency, or other hot Rust-only paths just to reduce size. Prefer type erasure at naturally expensive boundaries such as JS callbacks, N-API calls, async promise handling, configuration parsing, and plugin interop.

## Duplicate Dependency Review

Run `duplicate-deps.mjs` to capture duplicate versions, feature trees, and the current `deny.toml` duplicate-dependency exceptions for `rspack_node` by default.

The script writes:

- `duplicate-package-versions.tsv` for all duplicate package names found by `cargo metadata --all-features --locked` by default
- `duplicate-package-versions-with-deny.tsv` to show which duplicates are currently covered by `[bans].skip` or `[bans].skip-tree`
- `deny-skip-status.tsv` for every duplicate-dependency exception in `deny.toml`
- `deny-skip-remove-candidates.tsv` for skip entries that no longer match a current duplicate and should be removed from `deny.toml`

Review duplicate versions for:

- multiple versions of the same utility crate
- transitive duplicates caused by feature choices
- crates pulled in only by optional debug, tracing, rsdoctor, rstest, rslib, browser, loader, or builtin plugin paths
- `napi`, `napi-derive`, `napi-sys`, `tokio`, `serde`, SWC, lightningcss, and resolver-related crates

When duplicate dependencies are found, prefer workspace unification, feature narrowing, or feature-gating over local one-off dependency changes. If a duplicate is fixed, remove the matching `[bans].skip` or `[bans].skip-tree` entry from `deny.toml` in the same change. Treat `deny-skip-remove-candidates.tsv` as the cleanup checklist. The deny cleanup check uses `cargo metadata --all-features --locked` by default to match the CI `cargo deny --all-features check license bans` behavior; set `DENY_METADATA_ALL_FEATURES=0` only when intentionally doing a narrower local investigation.

## Optimization Patterns

Extract a shared object when many generated implementations repeat the same storage, registration, sorting, tracing, error wrapping, or conversion scaffolding. Keep the type-specific logic small and delegate the common work to a non-generic helper.

Use a shared trampoline when many generated N-API methods or hook wrappers repeat callback setup, argument count checks, panic/error conversion, async scheduling, or return conversion. The generated code should create a small descriptor and call a common trampoline; the descriptor can hold function pointers or a compact vtable.

Use dyn trait or type erasure when the dispatch boundary is already cold or already dominated by JS/N-API cost. Good candidates are JS plugin taps, `ThreadsafeFunction<T, R>` adapters, config callbacks, raw option parsing, stats or rsdoctor collectors, debug APIs, and binding-layer helpers that immediately cross into JavaScript. In these places, replacing many `T/R` monomorphized adapters with one erased adapter often saves code size without measurable runtime cost.

When extracting a common helper for a hot path, avoid pushing boxed trait objects or rich generic values through the helper if primitive data is enough. Prefer primitive proxies such as `usize` indexes, `i32` stages, enum tags, offsets, compact descriptors, or function pointers. This keeps the shared helper non-generic while preserving the fast path. PR #14633 is the reference pattern: hook metadata and base tap stages move into shared `HookCommon`, interceptor slow paths sort and merge lightweight stage/index data, and no-interceptor hook calls keep directly iterating taps.

Use stubs for disabled feature groups. If a builtin plugin, loader, debug API, or integration is optional, keep the public error surface but compile out the heavy implementation behind a feature. The stub should return a clear unsupported-in-this-build error.

Safe zones for dyn trait or type-erased adapters:

- JS plugin taps and N-API callbacks
- threadsafe function wrappers
- raw option parsing and config callbacks
- debug, stats, rsdoctor, and other non-default diagnostic paths
- binding-layer method dispatch where the N-API boundary dominates

Avoid dyn trait in tight Rust-only loops:

- parser hooks
- dependency and module graph traversals
- code generation inner loops
- hashing and cache key construction
- hot plugin hooks that stay entirely in Rust

## Review Heuristics

When reviewing a size increase, look for these signals:

- a new `#[napi(object)]` DTO without `object_from_js = false` or `object_to_js = false`
- new `Either*` or `ThreadsafeFunction<T, R>` combinations
- new raw builtin option structs added unconditionally
- a new dependency in `rspack_binding_api` that pulls a plugin, loader, parser, or large transform crate into `rspack_node`
- new macro-generated wrappers in files that already have many wrappers
- new feature flags that are additive but never used to remove code

Report the likely cause, the data that supports it, and the safest optimization path. Prefer concrete suggestions such as "make this DTO input-only", "move this branch behind a feature with a stub", "reuse the existing common runtime module helper", "type-erase this JS callback", or "replace repeated wrappers with a shared trampoline".

## Validation

For code changes that target binary size:

- build the relevant binding target before and after
- compare file size and section size when possible
- rerun the collection scripts
- run focused tests for changed binding behavior
- avoid storage and native watcher tests in sandbox unless explicitly requested

For pure analysis or review, include the report paths and summarize the most actionable growth sources first.
