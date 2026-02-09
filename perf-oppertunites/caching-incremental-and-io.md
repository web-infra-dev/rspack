# Caching, Incremental, and IO Opportunities

This document focuses on caching layers, incremental passes, and file-system IO.

## Where the Work Happens

- Cache interface: `crates/rspack_core/src/cache/mod.rs`
- Persistent cache: `crates/rspack_core/src/cache/persistent/**`
- Storage backend: `crates/rspack_storage/**`
- Incremental mutations: `crates/rspack_core/src/incremental/**`
- File system abstraction: `crates/rspack_fs/**`

## Optimization Opportunities

### 1) Cache write batching and async flush

Persistent cache storage can reduce build time, but writing on the critical
path can block:

- Ensure cache writes are batched and offloaded to async tasks.
- Use a single background flush per compilation pass instead of per entry.

Code pointers:

- `crates/rspack_storage/src/lib.rs`
- `crates/rspack_core/src/cache/persistent/**`

### 2) Reduce serialization allocations

`rspack_cacheable` uses `rkyv` and custom serializers. Opportunities:

- Use scratch buffers to reduce allocations for repeated serialization.
- Prefer `Vec::with_capacity` for predictable payload sizes.

### 3) Incremental mutation granularity

Incremental passes rely on mutation tracking:

- Reduce mutation set sizes by collapsing similar mutations.
- Use `IdentifierSet` diffs rather than full set recomputation.
- Avoid invalidating codegen cache when only metadata changes.

### 4) Snapshot strategy tuning

`cache/persistent/snapshot` controls when rebuilds occur:

- Use file size + mtime heuristics before full content hashes.
- Batch file stat calls to reduce IO overhead.

### 5) IO concurrency controls

The file system abstraction is used heavily for resolution and loader input:

- Add per‑stage IO concurrency limits to avoid disk contention.
- Cache `ReadFileSystem` results for repeated identical reads in a build.

### 6) Cache invalidation shortcuts

Some passes can skip work entirely if cache metadata shows no changes:

- Skip `module_graph` updates when entrypoints and dependencies are unchanged.
- Short‑circuit chunk graph rebuild when module hashes are identical.

### 7) Incremental pass disablement triggers

Some passes are disabled when full-hash dependencies or hash placeholders are
used (e.g. in `create_chunk_assets` and `create_hash`). Documenting and
minimizing these triggers can keep incremental benefits:

- Prefer stable filename templates that do not require full compilation hash.
- Avoid `dependent_full_hash` in plugins unless necessary.
