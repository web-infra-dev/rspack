# Chunking, Codegen, and Runtime

This document covers the chunk graph build, code generation, and runtime module
emission paths.

## Where the Work Happens

- Chunk graph build: `crates/rspack_core/src/compilation/build_chunk_graph/mod.rs`
- Code generation: `crates/rspack_core/src/compilation/code_generation/mod.rs`
- Chunk hashing: `crates/rspack_core/src/compilation/create_hash/mod.rs`
- Module hashing: `crates/rspack_core/src/compilation/create_module_hashes/mod.rs`
- Chunk asset rendering: `crates/rspack_core/src/compilation/create_chunk_assets/mod.rs`
- Process assets hook: `crates/rspack_core/src/compilation/process_assets/mod.rs`
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

`create_chunk_assets` calls render manifests per chunk, then emits assets. Each
chunk generates its own manifest and diagnostics collections. Opportunities:

- Batch rendering for chunks that share runtime configuration.
- Use shared `Vec` buffers for manifests/diagnostics to reduce allocation.
- Write assets in batches (group IO per output directory).

### 7) Hashing pass consolidation

`create_hash` and `create_module_hashes` can be expensive, especially when
incremental passes are disabled by full-hash dependencies:

- Avoid plugins that require `dependent_full_hash` unless strictly necessary.
- Cache `ChunkHashResult` and runtime module hashes across builds.
- For modules with identical runtime sets, reuse hash computations.

### 8) Process assets hook overhead

`process_assets` is plugin‑heavy. When many plugins tap, the overhead can grow:

- Use stage filtering so plugins only run when needed.
- Reduce asset cloning by passing references where possible.
- Aggregate asset updates to reduce repeated hash invalidations.

### 9) ReplaceSource usage in JS plugins

`ReplaceSource` appears in JS plugin code (e.g. URLPlugin). Opportunities:

- Avoid `into_string_lossy()` on large sources; operate on bytes or slices.
- Cache regex results for repeated placeholder patterns.
- Replace in chunks rather than cloning entire sources when possible.

Code pointer:

- `crates/rspack_plugin_javascript/src/plugin/url_plugin.rs`
