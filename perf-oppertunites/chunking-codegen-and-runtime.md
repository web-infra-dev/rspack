# Chunking, Codegen, and Runtime

This document covers the chunk graph build, code generation, and runtime module
emission paths.

## Where the Work Happens

- Chunk graph build: `crates/rspack_core/src/compilation/build_chunk_graph/mod.rs`
- Code generation: `crates/rspack_core/src/compilation/code_generation/mod.rs`
- Runtime modules: `crates/rspack_plugin_runtime/**`
- Split chunks: `crates/rspack_plugin_split_chunks/**`
- Runtime chunk: `crates/rspack_plugin_runtime_chunk/**`

## Optimization Opportunities

### 1) Re-enable incremental chunk graph (currently disabled)

`build_chunk_graph` explicitly disables incremental updates:

```rust
let enable_incremental = false;
```

Re‑introducing incremental chunk graph updates could remove full rebuilds for
minor changes. This should be validated for correctness, then gated behind a
feature flag to measure perf impact.

### 2) Reduce codegen job duplication

`code_generation_modules` builds a `HashMap<RspackHashDigest, CodeGenerationJob>`
per module, then expands jobs for each runtime:

- Reuse the map between modules to avoid allocations.
- Cache `runtime_template` and `ModuleCodeGenerationContext` instances.
- Avoid `hash.clone()` in tight loops where possible (use `Arc`).

### 3) Better runtime requirements caching

Runtime requirements are collected and then merged into codegen results:

```rust
codegen_res.runtime_requirements.extend(*runtime_template.runtime_requirements());
```

Opportunities:
- Reuse computed runtime requirements for identical runtime templates.
- Use bitsets for common runtime requirement sets to minimize hashing.

### 4) SplitChunks compute costs

`rspack_plugin_split_chunks` can be a major cost for large graphs:

- Cache group computation results by module/chunk signature.
- Use pre‑computed size thresholds to avoid per‑module scanning.
- Break long passes into parallel subpasses with bounded concurrency.

### 5) Runtime module template rendering

Runtime modules often render JS templates repeatedly:

- Cache template fragments keyed by runtime & feature flags.
- Avoid string concatenation in loops; use `String::with_capacity`.

### 6) Asset emission batching

Codegen output feeds into asset emission. Emitters can batch file writes and
hashing to reduce IO overhead, especially for many chunks.
