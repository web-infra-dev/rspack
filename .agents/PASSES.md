# Compilation passes

Read this before changing compilation flow, pass ordering, incremental
artifacts, or pass-local cache behavior.

## Source of truth

`Compilation::run_passes` in
`crates/rspack_core/src/compilation/run_passes.rs` defines the pass order.
Individual pass modules live under `crates/rspack_core/src/compilation/`.

## Composite phases

`BuildModuleGraphPhasePass` is a composite phase implemented in
`crates/rspack_core/src/compilation/build_module_graph/pass.rs`. It runs:

1. `make_hook_pass`
2. `build_module_graph_pass`
3. `finish_make_pass`
4. `finish_module_graph_pass`

`FinishModulesPhasePass` is implemented in
`crates/rspack_core/src/compilation/finish_modules/mod.rs`.

## Current pass order

`Compilation::run_passes` currently runs:

1. `BuildModuleGraphPhasePass`
2. `FinishModulesPhasePass`
3. `SealPass`
4. `OptimizeDependenciesPass`
5. `BuildChunkGraphPass`
6. `OptimizeModulesPass`
7. `OptimizeChunksPass`
8. `OptimizeTreePass`
9. `OptimizeChunkModulesPass`
10. `ModuleIdsPass`
11. `ChunkIdsPass`
12. `AssignRuntimeIdsPass`
13. `OptimizeCodeGenerationPass`
14. `CreateModuleHashesPass`
15. `CodeGenerationPass`
16. `RuntimeRequirementsPass`
17. `CreateHashPass`
18. `CreateModuleAssetsPass`
19. `CreateChunkAssetsPass`
20. `ProcessAssetsPass`
21. `AfterProcessAssetsPass`
22. `AfterSealPass`

`module_static_cache.enable_new_cache()` runs before the loop and
`module_static_cache.disable_cache()` runs after all passes complete.

## Change rules

- Moving work between passes can change incremental recovery semantics. Check
  `.agents/ARTIFACTS.md`.
- Adding a pass may require a new `IncrementalPasses` flag if recovered state is
  tied to the pass.
- Cache callbacks around graph-related phases live in the pass implementations,
  not only in `run_passes.rs`.
- Keep pass names and documentation aligned with `PassExt::name()`.
