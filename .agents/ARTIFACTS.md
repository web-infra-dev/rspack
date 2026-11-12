# Artifacts

Design and usage of the artifact system in Rspack's incremental compilation.

## Overview

Artifacts are data structures that hold intermediate compilation results. During incremental
compilation, they can be recovered into the next compilation so the associated pass updates only
the work affected by known mutations instead of recomputing all prior state.

Artifacts belong to Incremental compilation, not to Cache. `Compiler::incremental_artifacts` owns
the previous compilation state used for artifact recovery, independently from `cache` configuration
and from the selected Cache backend. See [Cache and Incremental Compilation](./CACHE_AND_INCREMENTAL.md)
for the complete ownership model.

Use **cache entry** or **cache value** for data whose lifecycle is controlled by Cache keys,
validation, storage, and eviction. Do not use Artifact as a generic synonym for cached data.

## Core concepts

### PASS binding rule

An artifact's `PASS` must be bound to the pass where that artifact is first built in the compilation pipeline.

- If an artifact is first produced in pass `X`, set `ArtifactExt::PASS` to `X`.
- Do not use `empty` as a fallback.
- If an artifact is read in later passes, it still stays bound to its first-build pass.

### ArtifactExt trait

The `ArtifactExt` trait is the foundation of the artifact system. It associates each artifact with its corresponding incremental pass and provides recovery logic.

```rust
pub trait ArtifactExt: Sized {
  /// The incremental pass associated with this artifact.
  const PASS: IncrementalPasses;

  /// Determines whether this artifact should be recovered from the previous compilation.
  fn should_recover(incremental: &Incremental) -> bool {
    incremental.mutations_readable(Self::PASS)
  }

  /// Recovers the artifact from the old compilation to the new compilation.
  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if Self::should_recover(incremental) {
      mem::swap(new, old);
    }
  }
}
```

### recover_artifact Function

A helper function that invokes the trait's recovery method:

```rust
pub fn recover_artifact<T: ArtifactExt>(incremental: &Incremental, new: &mut T, old: &mut T) {
  T::recover(incremental, new, old);
}
```

## Artifact types

### Representative direct artifacts

Examples of artifacts that directly implement `ArtifactExt`:

| Artifact                          | PASS                           | Description                     |
| --------------------------------- | ------------------------------ | ------------------------------- |
| `ModuleIdsArtifact`               | `MODULE_IDS`                   | Module ID mappings              |
| `ChunkNamedIdArtifact`            | `CHUNK_IDS`                    | Named chunk ID mappings         |
| `CgmHashArtifact`                 | `MODULES_HASHES`               | Module hash data                |
| `CgmRuntimeRequirementsArtifact`  | `MODULES_RUNTIME_REQUIREMENTS` | Module runtime requirements     |
| `CgcRuntimeRequirementsArtifact`  | `CHUNKS_RUNTIME_REQUIREMENTS`  | Chunk runtime requirements      |
| `ChunkHashesArtifact`             | `CHUNKS_HASHES`                | Chunk hash data                 |
| `ChunkRenderArtifact`             | `CHUNK_ASSET`                  | Chunk render results            |
| `CodeGenerationResults`           | `MODULES_CODEGEN`              | Code generation results         |
| `SideEffectsOptimizeArtifact`     | `OPTIMIZE_DEPENDENCIES`        | Side effects optimization data  |
| `AsyncModulesArtifact`            | `FINISH_MODULES`               | Async modules information       |
| `DependenciesDiagnosticsArtifact` | `FINISH_MODULES`               | Dependencies diagnostics        |
| `ImportedByDeferModulesArtifact`  | `OPTIMIZE_CHUNK_MODULES`       | Deferred module import tracking |

### Generation-Aware Artifacts

The following historical `*CacheArtifact` types have custom `recover` implementations that call
`start_next_generation()`:

| Artifact                                  | PASS                           | Description                |
| ----------------------------------------- | ------------------------------ | -------------------------- |
| `ChunkRenderCacheArtifact`                | `CHUNK_ASSET`                  | Chunk render cache         |
| `CodeGenerateCacheArtifact`               | `MODULES_CODEGEN`              | Code generation cache      |
| `ProcessRuntimeRequirementsCacheArtifact` | `MODULES_RUNTIME_REQUIREMENTS` | Runtime requirements cache |

These are transitional co-located Cache structures, not the preferred shape for new Incremental
artifacts. Their storage is influenced by Cache configuration even though they currently live on
`Compilation` and move with Incremental artifacts. New Cache-owned state should live behind Cache
abstractions instead of adding another `*CacheArtifact` type.

### Wrapper types

Wrapper types that delegate to the inner type's `PASS`:

| Wrapper          | Description                                    |
| ---------------- | ---------------------------------------------- |
| `StealCell<T>`   | Movable artifact wrapper used by `Compilation` |
| `BindingCell<T>` | JS binding-aware wrapper (napi feature)        |
| `Box<T>`         | Simple box wrapper (sys binding)               |

## Usage in rebuild

During rebuild, `Compiler` moves the completed compilation into its independent
`IncrementalArtifacts` holder:

```rust
let old_compilation = std::mem::replace(&mut self.compilation, next_compilation);
self
  .incremental_artifacts
  .store_previous_compilation(Box::new(old_compilation));
```

The pass runner then recovers only the artifacts bound to the pass before executing it and captures
special snapshots after a successful pass:

```rust
let incremental_passes = pass.incremental_passes();
incremental_artifacts.recover(incremental_passes, compilation);
pass.before_pass(compilation, cache).await;
let result = pass.run_pass_with_cache(compilation, cache).await;
if result.is_ok() {
  incremental_artifacts.capture(incremental_passes, compilation);
  pass.after_pass(compilation, cache).await;
}
```

Artifact recovery and Cache hooks are deliberately separate operations. `cache: false` changes the
Cache hooks but must not prevent `IncrementalArtifacts::recover` from running.

## Implementing a new artifact

### Basic artifact

```rust
use crate::{ArtifactExt, incremental::IncrementalPasses};

#[derive(Debug, Default)]
pub struct MyArtifact {
  // artifact data
}

impl ArtifactExt for MyArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::MY_PASS;
}
```

### Artifact with custom recovery

```rust
impl ArtifactExt for MyArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::MY_PASS;

  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if Self::should_recover(incremental) {
      *new = std::mem::take(old);
      new.prepare_for_rebuild();
    }
  }
}
```

Custom recovery must preserve the `should_recover` gate so `incremental: false` cannot move prior
compilation state into the new compilation.

### Wrapped artifact

For artifacts wrapped in `StealCell<T>`, `BindingCell<T>`, or `Box<T>`, the wrapper delegates to the
inner type's `PASS`.

```rust
// In Compilation struct
pub my_artifact: StealCell<MyArtifact>,

// Recovery is automatic through the wrapper's ArtifactExt impl
recover_artifact(
  incremental,
  &mut new_compilation.my_artifact,
  &mut self.compilation.my_artifact,
);
```

## Incremental passes

Incremental passes are bitflags that control which compilation phases may recover and reuse
artifacts. The source of truth is `crates/rspack_core/src/incremental/mod.rs`; do not copy the bit
values into documentation because the pass set evolves. Each `PassExt` declares its associated
passes through `incremental_passes()`.

## Design principles

1. **Separation of Concerns**: Artifacts belong to Incremental, while cache entries belong to Cache
2. **Automatic Recovery**: Wrapper types delegate recovery to inner types
3. **Custom Recovery**: Artifacts can override `recover` when swap semantics are insufficient, while preserving the Incremental gate
4. **Type Safety**: The trait system ensures compile-time correctness
5. **Performance**: `mem::swap` provides zero-copy artifact transfer

## File locations

- Incremental artifact owner: `crates/rspack_core/src/artifacts/incremental_artifacts.rs`
- Pass recovery wrapper: `crates/rspack_core/src/compilation/pass.rs`
- Rebuild lifecycle: `crates/rspack_core/src/compiler/rebuild.rs`
- Individual artifacts: `crates/rspack_core/src/artifacts/*.rs`
