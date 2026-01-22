# Artifacts

Design and usage of the artifact system in Rspack's incremental compilation.

## Overview

Artifacts are data structures that hold intermediate compilation results. They are designed to be recovered across rebuilds during incremental compilation, avoiding redundant recomputation when their associated compilation pass hasn't changed.

## Core Concepts

### ArtifactExt Trait

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

## Artifact Types

### Direct Artifacts

Artifacts that directly implement `ArtifactExt`:

| Artifact                          | PASS                           | Description                     |
| --------------------------------- | ------------------------------ | ------------------------------- |
| `ModuleIdsArtifact`               | `MODULES_IDS`                  | Module ID mappings              |
| `ChunkNamedIdArtifact`            | `CHUNKS_IDS`                   | Named chunk ID mappings         |
| `CgmHashArtifact`                 | `MODULES_HASHES`               | Module hash data                |
| `CgmRuntimeRequirementsArtifact`  | `MODULES_RUNTIME_REQUIREMENTS` | Module runtime requirements     |
| `CgcRuntimeRequirementsArtifact`  | `CHUNKS_RUNTIME_REQUIREMENTS`  | Chunk runtime requirements      |
| `ChunkHashesArtifact`             | `CHUNKS_HASHES`                | Chunk hash data                 |
| `ChunkRenderArtifact`             | `CHUNKS_RENDER`                | Chunk render results            |
| `CodeGenerationResults`           | `MODULES_CODEGEN`              | Code generation results         |
| `SideEffectsOptimizeArtifact`     | `SIDE_EFFECTS_OPTIMIZATION`    | Side effects optimization data  |
| `AsyncModulesArtifact`            | `INFER_ASYNC_MODULES`          | Async modules information       |
| `DependenciesDiagnosticsArtifact` | `DEPENDENCIES_DIAGNOSTICS`     | Dependencies diagnostics        |
| `ImportedByDeferModulesArtifact`  | empty                          | Deferred module import tracking |

### Cache Artifacts

Artifacts with custom `recover` implementations that call `start_next_generation()`:

| Artifact                                  | PASS                           | Description                |
| ----------------------------------------- | ------------------------------ | -------------------------- |
| `ChunkRenderCacheArtifact`                | `CHUNKS_RENDER`                | Chunk render cache         |
| `CodeGenerateCacheArtifact`               | `MODULES_CODEGEN`              | Code generation cache      |
| `ProcessRuntimeRequirementsCacheArtifact` | `MODULES_RUNTIME_REQUIREMENTS` | Runtime requirements cache |

### Wrapper Types

Wrapper types that delegate to the inner type's `PASS`:

| Wrapper                 | Description                                   |
| ----------------------- | --------------------------------------------- |
| `DerefOption<T>`        | Optional artifact wrapper with deref support  |
| `Arc<AtomicRefCell<T>>` | Shared artifact wrapper for concurrent access |
| `BindingCell<T>`        | JS binding-aware wrapper (napi feature)       |
| `Box<T>`                | Simple box wrapper (sys binding)              |

## Usage in Rebuild

During rebuild, artifacts are recovered from the old compilation to the new compilation:

```rust
// In Compiler::rebuild_inner

// Wrapped artifacts
recover_artifact(
  incremental,
  &mut new_compilation.async_modules_artifact,
  &mut self.compilation.async_modules_artifact,
);
recover_artifact(
  incremental,
  &mut new_compilation.code_generation_results,
  &mut self.compilation.code_generation_results,
);

// Direct type artifacts
recover_artifact(
  incremental,
  &mut new_compilation.module_ids_artifact,
  &mut self.compilation.module_ids_artifact,
);
```

## Implementing a New Artifact

### Basic Artifact

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

### Cache Artifact with Custom Recovery

```rust
impl ArtifactExt for MyCacheArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::MY_PASS;

  fn recover(_incremental: &Incremental, new: &mut Self, old: &mut Self) {
    *new = std::mem::take(old);
    new.start_next_generation();
  }
}
```

### Wrapped Artifact

For artifacts wrapped in `Arc<AtomicRefCell<T>>`, `DerefOption<T>`, or `BindingCell<T>`, the wrapper automatically delegates to the inner type's `PASS`.

```rust
// In Compilation struct
pub my_artifact: Arc<AtomicRefCell<MyArtifact>>,

// Recovery is automatic through the wrapper's ArtifactExt impl
recover_artifact(
  incremental,
  &mut new_compilation.my_artifact,
  &mut self.compilation.my_artifact,
);
```

## Incremental Passes

Incremental passes are bitflags that control which compilation phases are enabled:

```rust
bitflags! {
  pub struct IncrementalPasses: u32 {
    const MAKE = 0b0000_0001;
    const MODULES_IDS = 0b0000_0010;
    const CHUNKS_IDS = 0b0000_0100;
    const MODULES_HASHES = 0b0000_1000;
    const MODULES_CODEGEN = 0b0001_0000;
    const MODULES_RUNTIME_REQUIREMENTS = 0b0010_0000;
    const CHUNKS_RUNTIME_REQUIREMENTS = 0b0100_0000;
    const CHUNKS_HASHES = 0b1000_0000;
    const CHUNKS_RENDER = 0b0001_0000_0000;
    // ... additional passes
  }
}
```

## Design Principles

1. **Separation of Concerns**: Each artifact is associated with related incremental pass
2. **Automatic Recovery**: Wrapper types delegate recovery to inner types
3. **Custom Recovery**: Cache artifacts can override `recover` for generation management
4. **Type Safety**: The trait system ensures compile-time correctness
5. **Performance**: `mem::swap` provides zero-copy artifact transfer

## File Locations

- Rebuild logic: `crates/rspack_core/src/compiler/rebuild.rs`
- Individual artifacts: `crates/rspack_core/src/artifacts/*.rs`
