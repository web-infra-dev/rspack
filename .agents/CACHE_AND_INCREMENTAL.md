# Cache and Incremental Compilation

This document defines the boundary between Rspack's build cache and incremental compilation. They
are independent performance mechanisms and must remain independently configurable.

## Canonical Terms

| Term                    | Meaning                                                                                                                                              |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| Cache                   | Fine-grained memoization of build computations. Cache entries are addressed by cache keys and validity data such as etags or snapshots.              |
| Memory cache            | Cache entries retained in the current compiler process. They do not survive process exit.                                                            |
| Filesystem cache        | Persistent cache entries stored on disk and reusable by later compiler processes. The public configuration calls this `type: 'persistent'`.          |
| Incremental compilation | Development-time rebuild logic that reuses unaffected intermediate compilation state and recomputes only work affected by a known change set.        |
| Incremental artifact    | Pass-owned intermediate compilation state that can move from the previous compilation to the next incremental compilation.                           |
| Cache entry             | A value owned and invalidated by the Cache subsystem. A cache entry is not an Incremental artifact, even when it contains similar intermediate data. |
| Cache backend           | The internal implementation of the Cache contract. Rspack currently has `legacy_cache` and `new_cache` backends.                                     |

Use these terms by ownership, not by the shape of the data. In particular, **Artifact is not a
synonym for cached value**. New Cache-owned payloads should be called cache entries or cache values,
and new Incremental-owned intermediate state should be modeled as artifacts.

## Two Orthogonal Configuration Axes

`cache` and `incremental` are orthogonal axes:

- `cache` decides whether fine-grained cache entries may be read and written, and which storage mode
  backs those entries.
- `incremental` decides whether a development rebuild may recover prior pass artifacts from the
  previous compilation, then update affected work from the known mutations.

Neither option implicitly enables or disables the other. All four combinations are valid:

| Cache | Incremental | Meaning                                                                                                                                                                                          |
| ----- | ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Off   | Off         | No state from a prior compilation or build is reused through Cache or Incremental. This is useful for debugging and full-rebuild test coverage. Compilation-local optimizations may still apply. |
| On    | Off         | Incremental artifact recovery is disabled, but individual computations may still hit memory or filesystem cache entries.                                                                         |
| Off   | On          | Development rebuilds recover prior pass artifacts so unaffected portions can be reused, but no fine-grained Cache entries are read or written.                                                   |
| On    | On          | Development rebuilds combine prior-artifact recovery with fine-grained cache hits. This is the normal high-performance development configuration.                                                |

In this table, Cache “On” includes either memory-only cache or filesystem-backed persistent cache.
The choice of storage mode does not change the Incremental semantics.

### Configuration Examples

```js
// Cache off, Incremental on
export default {
  cache: false,
  incremental: true,
};
```

```js
// Filesystem cache on, Incremental off
export default {
  cache: {
    type: 'persistent',
  },
  incremental: false,
};
```

## Incremental Is a Development Rebuild Mechanism

Incremental compilation is designed for rebuilds performed by the same compiler with a known set of
modified and removed files. The supported user-facing workflows are development, watch mode, and
HMR.

Incremental passes are enabled only when `mode` is `development`. The initial compilation prepares
artifacts for a possible later rebuild, even though it cannot reuse a previous compilation.

A standalone `rspack build` creates a fresh compiler and performs a one-shot build. There is no
previous in-memory compilation to incrementally update, so `incremental` does not turn separate
`rspack build` invocations into incremental builds. A later standalone build invocation can still
become faster through Cache, especially when the filesystem cache restores fine-grained entries
from a previous process; that is a **cache hit**, not an incremental build.

Rspack's `incremental` option is conceptually aligned with webpack's
[`cacheUnaffected`](https://webpack.js.org/configuration/cache/#cachecacheunaffected) semantics: keep
the computation of unaffected modules and compilation stages reusable while rebuilding after a
change. The alignment is semantic, not structural. Rspack exposes Incremental as an independent
top-level option and does not inherit webpack's configuration coupling between `cacheUnaffected` and
memory cache.

## Cache Storage Modes

Cache supports fine-grained entries through two storage modes:

1. **Memory cache**: `cache: true` or `cache: { type: 'memory' }`. Entries can be reused by
   compilations in the current compiler process.
2. **Filesystem cache**: `cache: { type: 'persistent' }` with filesystem storage. Entries survive
   process exit and can accelerate later builds after validation.

Filesystem-backed cache implementations may keep a memory front cache. That implementation detail
does not create a third public storage mode.

## Persistent Cache Format Compatibility

Rspack's persistent cache is internal derived data, not a cross-version storage format. Cache data
written by one Rspack version does not need to remain readable by another Rspack version.

When a serialized cache value, key meaning, database family, or metadata schema changes, invalidate
the incompatible cache through the existing package/cache version validation and rebuild it. Prefer
version invalidation and database reset over migration code, fallback decoding, dual-format writes,
or retaining obsolete fields solely to read cache data from older Rspack versions.

This rule does not relax correctness within one version: every cache entry and its required metadata
must still be written, validated, restored, and invalidated consistently.

## Cache Backends

Rspack currently carries two implementations of the same Cache responsibility:

| Backend        | Location                               |
| -------------- | -------------------------------------- |
| `legacy_cache` | `crates/rspack_core/src/legacy_cache/` |
| `new_cache`    | `crates/rspack_core/src/new_cache/`    |

The backend selector is not another Incremental mode. Switching between `legacy_cache` and
`new_cache` may change how cache entries are keyed, retained, serialized, or flushed, but it must not
change which Incremental passes are enabled or whether Incremental artifacts can be recovered.

During the backend migration, `Compiler` contains handles for both implementations and disables the
unselected one. Code should depend on their Cache responsibility rather than use either backend as
the owner of Incremental state. The two implementations do not need identical cache coverage or
storage formats while the migration is in progress.

## Ownership and Data Flow

Ownership is the architectural boundary:

| State                                                | Owner                                | Lifetime                                                                |
| ---------------------------------------------------- | ------------------------------------ | ----------------------------------------------------------------------- |
| Current pass artifacts                               | `Compilation`                        | One compilation, unless recovered into the next development compilation |
| Previous-compilation artifacts and special snapshots | `IncrementalArtifacts` on `Compiler` | Across compilations in the same compiler                                |
| Fine-grained cache entries                           | Cache backend                        | Compiler-process memory and optionally filesystem persistence           |
| Compilation-local memoization                        | `transient_cache`                    | One compilation only                                                    |

The two reuse paths are separate:

```text
Development rebuild

previous Compilation ──> IncrementalArtifacts ──> pass-bound recovery ──> next Compilation

fine-grained computation <── key / etag / validation ──> Cache <──> memory or filesystem
```

`IncrementalArtifacts` owns the previous compilation used by incremental passes. Before a pass runs,
the pass runner asks `IncrementalArtifacts` to recover the artifacts bound to that pass. The legacy
Cache's restore and save hooks run independently around the pass. Consumers of the new Cache use
`CacheFacade` instead; this path is also independent from Incremental recovery.

Some historical or transitional types do not yet follow the physical boundary. In particular,
`CodeGenerateCacheArtifact`, `ProcessRuntimeRequirementsCacheArtifact`, and
`ChunkRenderCacheArtifact` are stored on `Compilation` and moved by `IncrementalArtifacts`, while
their generation-aware memoization is controlled by Cache configuration. These types are not a
precedent for new code: ownership is determined by invalidation and reuse semantics, and Cache-owned
state should move behind Cache abstractions. Legacy persistent Cache calls its generic payload a
`CacheItem`, not an Artifact.

## Architectural Invariants

Changes to these systems must preserve the following rules:

1. `cache: false` must not clear or rewrite `IncrementalPasses`.
2. `incremental: false` must not disable Cache lookup or storage.
3. `legacy_cache` and `new_cache` must not own the previous compilation for Incremental recovery.
4. Incremental artifacts must be recovered by the compiler's Incremental path, not by Cache hooks.
5. Cache-owned data must be addressed, validated, stored, and evicted through Cache abstractions.
6. A filesystem cache hit during a one-shot build must be described as Cache acceleration, not as an
   incremental build.
7. Tests that require full-rebuild behavior must set `incremental: false` explicitly instead of
   relying on `cache: false` as an indirect switch.

Most pass-scoped Incremental state follows these rules today. The `*CacheArtifact` types described
above are transitional Cache co-location, while `EMIT_ASSETS` still keeps emitted asset versions
directly on `Compiler` instead of in an Incremental artifact. Neither exception should be used to
introduce new Cache/Incremental coupling.

## Relevant Code

- Cache configuration: `crates/rspack_core/src/options/cache.rs`
- Shared Cache dependencies: `crates/rspack_core/src/cache/`
- Legacy Cache backend: `crates/rspack_core/src/legacy_cache/`
- New Cache backend: `crates/rspack_core/src/new_cache/`
- Incremental configuration and mutations: `crates/rspack_core/src/incremental/`
- Incremental artifact ownership: `crates/rspack_core/src/artifacts/incremental_artifacts.rs`
- Pass recovery and legacy Cache hooks: `crates/rspack_core/src/compilation/pass.rs`
- Rebuild lifecycle: `crates/rspack_core/src/compiler/rebuild.rs`
- Artifact design: [`ARTIFACTS.md`](./ARTIFACTS.md)
