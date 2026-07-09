# Architecture map

This file is a map for agents, not a full architecture guide. Use it to decide
which code and deeper notes to read before changing Rspack.

## Repository shape

- Rust crates live under `crates/`.
- JavaScript and TypeScript packages live under `packages/`.
- Integration fixtures and test harnesses live under `tests/rspack-test/`.
- End-to-end tests live under `e2e/`.
- User-facing docs live under `website/docs/`.
- Agent-specific implementation notes live under `.agents/`.

## First files to read

- `AGENTS.md`: repo-wide build, test, concurrency, and PR rules.
- `CONTEXT.md`: canonical domain language for Rspack concepts.
- `.agents/CODE_STYLE.md`: formatting and linting facts derived from config.
- `.agents/PASSES.md`: compilation pass order.
- `.agents/ARTIFACTS.md`: incremental artifact recovery rules.
- `.agents/TRANSIENT_CACHE.md`: single-compilation cache semantics.

## Core Rust areas

- `crates/rspack_core/`: compiler, compilation, graphs, chunks, assets,
  incremental artifacts, runtime requirements, and shared abstractions.
- `crates/rspack_core/src/compilation/`: pass modules run by
  `Compilation::run_passes`.
- `crates/rspack_core/src/artifacts/`: artifact types that participate in
  incremental recovery.
- `crates/rspack_core/src/incremental/`: incremental pass flags and mutation
  tracking.
- `crates/rspack_core/src/transient_cache.rs`: caches that are intentionally
  scoped to one compilation lifecycle.

## Extension areas

- `crates/rspack_plugin_*/`: Rust implementations of built-in plugins.
- `crates/rspack_loader_*/`: Rust loader implementations and loader plugins.
- `packages/rspack/src/`: public JavaScript API surface.
- `packages/rspack-cli/src/`: CLI behavior.
- `packages/rspack-test-tools/` and `tests/rspack-test/`: test harness and
  compatibility fixtures.

## Compilation pipeline

`Compilation::run_passes` in `crates/rspack_core/src/compilation/run_passes.rs`
is the source of truth for pass order. Before changing compilation flow,
incremental recovery, or pass-local cache behavior, read:

- `.agents/PASSES.md`
- `.agents/ARTIFACTS.md`
- `.agents/TRANSIENT_CACHE.md`

Then inspect the relevant pass module under `crates/rspack_core/src/compilation/`.

## Public API layer

Public JavaScript APIs must preserve webpack compatibility unless the change
explicitly documents a compatibility gap. Before changing public API shape,
read:

- `.agents/API_DESIGN.md`
- `packages/rspack/src/`
- `website/docs/en/api/`
- Existing compatibility tests under `tests/rspack-test/`

## Concurrency boundary

Follow the concurrency rules in `AGENTS.md`:

- Use `rayon` for CPU-bound synchronous parallel work.
- Use `rspack_parallel` abstractions for async orchestration.
- Avoid mixing `rayon` and `tokio` thread pools inside one workflow without a
  clear boundary.

Do not introduce raw `tokio` orchestration for synchronous CPU-heavy work.

## Documentation boundary

Do not duplicate glossary definitions in `.agents/`. Domain language belongs in
`CONTEXT.md`. Agent notes should capture implementation facts that affect code
changes: pass order, cache semantics, artifact recovery, API constraints, and
where to find working examples.
