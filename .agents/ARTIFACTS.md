# Incremental artifacts

Read this before changing incremental recovery, compilation artifacts, or pass
cache behavior.

## Source of truth

- Artifact trait and recovery helper:
  `crates/rspack_core/src/artifacts/mod.rs`
- Incremental pass flags: `crates/rspack_core/src/incremental/mod.rs`
- Rebuild recovery flow: `crates/rspack_core/src/compiler/rebuild.rs`
- Artifact implementations: `crates/rspack_core/src/artifacts/*.rs`

## Binding rule

`ArtifactExt::PASS` must name the pass where the artifact is first built or
made valid.

- If an artifact is first produced in pass `X`, bind it to `X`.
- If later passes read that artifact, keep the original first-build pass.
- Do not bind an artifact to a later consumer just because that consumer is more
  visible.
- Do not use an empty or unrelated pass as a fallback.

This binding controls whether old compilation data can be recovered safely.

## Recovery model

The default `ArtifactExt::recover` swaps old artifact state into the new
compilation when `incremental.mutations_readable(Self::PASS)` is true.

Cache-like artifacts may override `recover` to move state and start a new
generation. Read the artifact implementation before assuming recovery is a plain
swap.

## Current artifact bindings

Keep this table aligned with `impl ArtifactExt` in `crates/rspack_core/src/artifacts/`.

| Artifact                                  | PASS                           |
| ----------------------------------------- | ------------------------------ |
| `BuildModuleGraphArtifact`                | `BUILD_MODULE_GRAPH`           |
| `ExportsInfoArtifact`                     | `BUILD_MODULE_GRAPH`           |
| `AsyncModulesArtifact`                    | `FINISH_MODULES`               |
| `DependenciesDiagnosticsArtifact`         | `FINISH_MODULES`               |
| `SideEffectsOptimizeArtifact`             | `OPTIMIZE_DEPENDENCIES`        |
| `BuildChunkGraphArtifact`                 | `BUILD_CHUNK_GRAPH`            |
| `ImportedByDeferModulesArtifact`          | `OPTIMIZE_CHUNK_MODULES`       |
| `ModuleIdsArtifact`                       | `MODULE_IDS`                   |
| `ChunkNamedIdArtifact`                    | `CHUNK_IDS`                    |
| `CgmHashArtifact`                         | `MODULES_HASHES`               |
| `CodeGenerationResults`                   | `MODULES_CODEGEN`              |
| `CodeGenerateCacheArtifact`               | `MODULES_CODEGEN`              |
| `CgmRuntimeRequirementsArtifact`          | `MODULES_RUNTIME_REQUIREMENTS` |
| `ProcessRuntimeRequirementsCacheArtifact` | `MODULES_RUNTIME_REQUIREMENTS` |
| `CgcRuntimeRequirementsArtifact`          | `CHUNKS_RUNTIME_REQUIREMENTS`  |
| `RuntimeProxyMetadataArtifact`            | `CHUNKS_RUNTIME_REQUIREMENTS`  |
| `ChunkHashesArtifact`                     | `CHUNKS_HASHES`                |
| `ChunkRenderArtifact`                     | `CHUNK_ASSET`                  |
| `ChunkRenderCacheArtifact`                | `CHUNK_ASSET`                  |

Not every type with `Artifact` in its name implements `ArtifactExt`. Check the
code before adding a table entry.

## Wrapper types

The artifact module provides wrapper implementations for:

- `Box<T>` when the `napi` feature is disabled.
- `BindingCell<T>` when the `napi` feature is enabled.

`StealCell<T>` delegates `ArtifactExt` to the wrapped type in
`crates/rspack_core/src/utils/steal_cell.rs`.

## Change checklist

When adding or moving an artifact:

1. Identify the first pass that builds or validates the artifact.
2. Bind `ArtifactExt::PASS` to that pass.
3. Decide whether default swap recovery is correct or whether generation state
   needs custom recovery.
4. Update this file if the artifact participates in recovery.
5. Add a regression test if an incorrect recovery could survive one rebuild and
   fail on a later rebuild.
