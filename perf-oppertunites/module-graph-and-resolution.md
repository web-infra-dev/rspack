# Module Graph & Resolution Opportunities

This document focuses on the module graph construction and resolution pipeline,
with pointers to the code paths that are likely to benefit from optimization.

## Where the Work Happens

**Module graph build & updates**
- `crates/rspack_core/src/compilation/build_module_graph/mod.rs`
- `crates/rspack_core/src/compilation/build_module_graph/graph_updater/mod.rs`
- `crates/rspack_core/src/module_graph/**`

**Rollback overlay map**
- `crates/rspack_core/src/module_graph/rollback/overlay_map.rs`

**Resolver**
- `crates/rspack_core/src/resolver/resolver_impl.rs`
- `crates/rspack_core/src/resolver/factory.rs`

## Hotspot Evidence

Perf samples show `OverlayMap::get` inside module graph rollback maps. This
means overlay lookups (used for incremental rebuilds) are occurring frequently
even in full builds.

## Optimization Opportunities

### 1) Reduce overlay churn in full builds

`OverlayMap` uses a base map plus optional overlay (`HashMap<K, OverlayValue<V>>`).
The `get` path checks the overlay first; in a full build that touches most
entries, it may be cheaper to operate directly on base maps without overlay:

- Avoid enabling overlay mode when no incremental rollback is expected.
- Consider exposing a fast path where `overlay.is_none()` is checked once per
  iteration rather than per lookup.

### 2) Avoid repeated `HashSet`/`Vec` allocations in build graph

`build_module_graph` allocates new `HashSet` and `Vec` inputs each pass:

```rust
params.push(UpdateParam::BuildEntry(
  compilation.entries.values().flat_map(...).collect(),
));
```

Opportunities:
- Reuse buffers between passes (retain capacity).
- Precompute entry dependency lists when entries do not change.
- Use `SmallVec`/`Vec` + dedup rather than repeated `HashSet` creation.

### 3) Reduce cloning in rollback `OverlayMap::get_mut`

`get_mut` materializes overlay values by cloning from base:

```rust
if let Some(value) = self.base.get(key).cloned() {
  self.overlay().insert(key.clone(), OverlayValue::Value(value));
}
```

Opportunities:
- Use `Arc<V>` for large values to avoid full clones.
- Store diffs instead of full clones when only small mutations occur.

### 4) Resolver allocation & path conversion churn

`Resolver::resolve` and `resolve_with_context` allocate new `String`s for
query/fragment and clone paths into `Utf8PathBuf`. Potential optimizations:

- Reuse `ResolveResult` buffers between calls (pooling).
- Avoid `to_string()` for query/fragment when empty; store `Option<Arc<str>>`.
- Cache `DescriptionData` results from package.json parsing.
- Aggregate resolution of related specifiers to share resolver cache hits.

### 5) Reduce error formatting on success paths

The resolver converts errors into rich diagnostics. Ensure error formatting
work happens only on failure. Any logging/formatting (cyan/yellow) should be
lazy and gated behind error handling paths.

### 6) Incremental update heuristics

`build_module_graph` uses `UpdateParam::CheckNeedBuild`, `ModifiedFiles`, and
`RemovedFiles`. Opportunities:

- Track per-module dependency hashes to avoid scanning all modified files.
- Use path prefix grouping to reduce repeated lookups in large `ArcPathSet`s.
- Consider a “dirty module” index to avoid recomputing `BuildEntry` on each pass.

