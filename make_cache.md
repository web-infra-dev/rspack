# Make-Stage Fine-Grained Cache Design

## Status

- Status: Draft
- Scope: Make / `BuildModuleGraph` with `experiments.newCache` enabled

See [Cache and Incremental Compilation](.agents/CACHE_AND_INCREMENTAL.md) for the canonical
terminology and the general relationship between Cache and Incremental. This document records the
Make-specific design decisions.

## Decisions

Cache entries and Incremental artifacts are independent:

- `cache` controls fine-grained module cache reads and writes.
- `incremental.buildModuleGraph` controls recovery of the previous compilation's in-memory
  `BuildModuleGraphArtifact` during same-compiler rebuilds.
- Neither option implicitly enables or disables the other.

The persistent cache stores module entries only. It never stores or restores the module graph or
other Incremental artifacts. Therefore:

- `incremental.buildModuleGraph: false` still allows module cache hits, but every compilation
  reconstructs the graph from current entries.
- `incremental.buildModuleGraph: true` may recover previous in-memory artifacts during a rebuild;
  module cache remains available for misses and process restarts.
- A process restart always reconstructs the graph. Filesystem cache can still avoid rebuilding
  valid individual modules.
- `cache: false` does not disable Incremental recovery.

The initial build and a full graph reconstruction still traverse modules and dependency edges. A
module cache hit avoids the module build, not the graph traversal or cache validation.

## Recovery mode

```text
incremental recovery = incremental.buildModuleGraph
                    && previous successful BuildModuleGraphArtifact exists

otherwise           = full graph reconstruction
```

Only artifacts from the previous successful compilation may be recovered. Full graph
reconstruction creates fresh artifacts and traverses current entries; valid module cache entries
can still turn individual builds into cache hits.

## Module-cache boundary

Both recovery modes use the same module-cache path. The build-graph code should only orchestrate
module hit/miss behavior and lifecycle hooks. The module-cache layer should own:

- lookup through the existing `Cache` / `CacheFacade` abstractions;
- memory and filesystem storage selection;
- value-dependency and file/context/missing-dependency snapshot validation;
- decoding, build-state restoration, uncacheable entries, and dependency ID handling;
- pending writes, including the dependency ID watermark needed by persisted entries.

The current implementation stores entries under the `Compilation/modules` namespace, keyed by a
stable module identifier. The entry contains the normal-module build state, dependencies, blocks,
optimization bailouts, and its module snapshot. The concrete serialization format should remain an
implementation detail of the cache layer.

## Hit and miss behavior

A module hit requires all of the following:

1. A cache entry exists and is reusable.
2. Value-cache dependencies are unchanged.
3. The file, context, and missing-dependency snapshot is valid.
4. The entry decodes and restores successfully.

Validation or compatibility misses rebuild the module. Cache/storage errors should be reported
through cache diagnostics and fall back to a module build; they must not fail Make merely because a
cache entry is unusable.

When a module builds successfully, its entry may be queued for persistence even if a later
compilation phase fails. In that case the dependency ID watermark must be persisted with the
pending entry; otherwise a later process can allocate IDs that collide with IDs embedded in the
restored cache entry.

## Invariants

1. Filesystem cache contains fine-grained module entries, never Incremental artifacts.
2. `incremental: false` does not disable cache lookup or storage.
3. `cache: false` does not clear or rewrite Incremental artifacts.
4. Only a previous successful `BuildModuleGraphArtifact` can be recovered incrementally.
5. Cache-owned data is addressed, validated, stored, and evicted through Cache abstractions.
6. A filesystem hit after process startup is cache acceleration, not an Incremental build.

## References

- Cache configuration: `crates/rspack_core/src/options/cache.rs`
- New Cache backend: `crates/rspack_core/src/new_cache/`
- Incremental artifacts: `crates/rspack_core/src/artifacts/incremental_artifacts.rs`
- Pass recovery: `crates/rspack_core/src/compilation/pass.rs`
- Make module cache: `crates/rspack_core/src/compilation/build_module_graph/graph_updater/repair/build.rs`
- Incremental configuration: `website/docs/en/config/incremental.mdx`
