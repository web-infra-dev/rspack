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

Here, “development-time” describes the long-lived rebuild workflow, not `mode: 'development'`.
Production-mode watch and other explicit rebuilds in the same compiler can also use Incremental.

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

## Incremental Compiler Contract

The native compiler has one Incremental switch: the resolved `incremental` option. Its meaning does
not depend on whether an individual call came from `run()` or `watch()`:

- Enabled passes mean that the compiler supports Incremental rebuilds. A cold build performs the
  bookkeeping needed by a later rebuild.
- No enabled passes mean that the compiler does not support Incremental rebuilds. Builds use the
  ordinary pass runner and create no Incremental checkpoints or artifacts.

The JavaScript compiler resolves this option once, before creating the native compiler. Watch mode
keeps the default Incremental passes enabled; a standalone run without an explicit Incremental
option resolves them to disabled. An explicit `incremental` option is preserved. After the native
compiler exists, later builds do not reinterpret the option based on their call site.

`incremental_ready` is not another enable switch. It records only whether the previous successful
compilation completed the bookkeeping required for hot recovery:

```text
bookkeeping = incremental.enabled()
hot_recovery = incremental.enabled() && incremental_ready
```

Readiness is cleared before a build starts and restored only after a successful Incremental
compilation. Therefore a failed or partial build cannot be treated as a valid recovery source; the
next rebuild falls back to cold Incremental bookkeeping.

### Cold and Hot Incremental Compilations

- An **ordinary compilation** has Incremental disabled and performs no bookkeeping.
- A **cold Incremental compilation** has Incremental enabled but no ready previous artifacts. It
  performs full work and captures artifacts.
- A **hot Incremental rebuild** has Incremental enabled and ready previous artifacts. It recovers
  pass-owned artifacts and applies mutations.

Only readiness changes between cold and hot compilations. Incremental enablement is fixed when the
native compiler is created.

### Pass and Artifact Conventions

- A pass declares its Incremental ownership through `incremental_passes`. Recovery and capture
  belong in the shared Incremental pass runner, not in Cache hooks or ad hoc compiler branches.
- Compilation-specific checkpoints must be guarded by the enabled pass set.
- A cold Incremental pass may capture artifacts but must never read mutations or recover artifacts
  from an unready compilation.
- A hot Incremental pass reads only the mutations and artifacts owned by its enabled pass group.
- Disabling an Incremental pass must also prevent its artifact recovery, checkpointing, and capture.
- Pass implementations must not inspect Cache enablement to decide whether Incremental state is
  available. They must use the Incremental lifecycle and pass APIs.
- Readiness belongs on `Compiler`; pass-owned reusable state belongs in `IncrementalArtifacts`;
  current compilation state belongs on `Compilation`.

### Testing Conventions

Lifecycle changes should use existing JavaScript integration test harnesses rather than crate-local
Rust unit tests. Cover at least:

- one-shot build with resolved Incremental disabled;
- one-shot build with Incremental explicitly enabled;
- initial watch compilation followed by at least one hot rebuild;
- explicitly requested same-compiler rebuilds outside watch mode;
- failure followed by a successful recovery build;
- `incremental: false` in watch mode; and
- representative Cache on/off combinations to ensure Cache does not control Incremental lifecycle.

## Cache Storage Modes

Cache supports fine-grained entries through two storage modes:

1. **Memory cache**: `cache: true` or `cache: { type: 'memory' }`. Entries can be reused by
   compilations in the current compiler process.
2. **Filesystem cache**: `cache: { type: 'persistent' }` with filesystem storage. Entries survive
   process exit and can accelerate later builds after validation.

Filesystem-backed cache implementations may keep a memory front cache. That implementation detail
does not create a third public storage mode.

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
8. A native compiler with Incremental disabled must not enter the Incremental artifact pipeline.
9. Hot artifact recovery requires both enabled Incremental passes and readiness from the previous
   successful compilation.
10. Readiness must not enable Incremental; it only selects hot recovery when Incremental is already
    enabled.

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
- Compiler lifecycle and Incremental activation: `crates/rspack_core/src/compiler/mod.rs`
- Rebuild lifecycle: `crates/rspack_core/src/compiler/rebuild.rs`
- JavaScript compiler lifecycle: `packages/rspack/src/Compiler.ts`
- Artifact design: [`ARTIFACTS.md`](./ARTIFACTS.md)
