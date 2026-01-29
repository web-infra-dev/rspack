# Compilation Passes Architecture

This document describes the modular architecture of the compilation process in rspack.

## Overview

The compilation process is organized into independent modules, each responsible for a specific phase or pass. This modular design improves code maintainability, testability, and allows for better separation of concerns.

## Module Structure

```
compilation/
├── mod.rs                        # Main Compilation struct which exposes the public API
├── run_passes.rs                 # Pass driver invoked from Compiler that runs make + seal passes
├── make/                         # make_hook_pass: make hook + cache.before_build_module_graph
├── build_module_graph/           # build_module_graph_pass: module graph construction
├── finish_make/                  # finish_make_pass: finish_make hook
├── finish_module_graph/          # finish_module_graph_pass: finalize module graph + cache
├── finish_modules/               # finish_modules_pass: finish_modules hook, diagnostics, checkpoint
├── seal/                         # seal_pass: seal hook
├── optimize_dependencies/        # optimizeDependencies hook + side effects artifact
├── build_chunk_graph/            # Chunk graph construction (code splitting cache + pass wrapper)
├── optimize_modules/             # optimizeModules + afterOptimizeModules hooks
├── optimize_chunks/              # optimizeChunks hook
├── optimize_tree/                # optimizeTree hook
├── optimize_chunk_modules/       # optimizeChunkModules hook
├── module_ids/                   # Module ID assignment + diagnostics
├── chunk_ids/                    # Chunk ID assignment + diagnostics
├── assign_runtime_ids/           # Runtime ID assignment for runtime chunks
├── optimize_code_generation/     # optimizeCodeGeneration hook
├── create_module_hashes/         # Module hash computation (incremental aware)
├── code_generation/              # Module codegen + afterCodeGeneration hook
├── runtime_requirements/         # Module/chunk/tree runtime requirements + runtime modules
├── create_hash/                  # Chunk hashing, runtime module hashes, full hash + runtime module codegen
├── create_module_assets/         # Emit module-declared assets and mark chunk auxiliary files
├── create_chunk_assets/          # Render manifests and emit chunk assets
├── process_assets/               # processAssets + afterProcessAssets hooks
└── after_seal/                   # afterSeal hook
```

## Pass Entry

- `Compiler::compile` builds `CompilationParams`, fires `thisCompilation` then `compilation` compiler hooks (binding safety for JS), and delegates to `Compilation::run_passes`.
- `Compilation::run_passes` performs the make and seal stages using the order below.

## Pass Order (Compilation::run_passes)

`run_passes` orchestrates the full pipeline (make + seal) in this order:

1. `make_hook_pass`: `make` hook + cache.before_build_module_graph
2. `build_module_graph_pass`: build module graph
3. `finish_make_pass`: `finish_make` hook
4. `finish_module_graph_pass`: `finish_build_module_graph` + cache.after_build_module_graph
5. `finish_modules_pass`: `finish_modules` hook, collect diagnostics, incremental checkpoint
6. Freeze module static cache in production
7. `seal_pass`: `seal` hook
8. `optimize_dependencies_pass`
9. `build_chunk_graph_pass` → `optimize_modules_pass` → `optimize_chunks_pass`
10. `optimize_tree_pass` → `optimize_chunk_modules_pass`
11. `module_ids_pass` → `chunk_ids_pass` → `assign_runtime_ids`
12. `optimize_code_generation_pass`
13. `create_module_hashes_pass`
14. `code_generation_pass`
15. `runtime_requirements_pass`
16. `create_hash_pass` (also runs runtime module code generation)
17. `create_module_assets_pass`
18. `create_chunk_assets_pass`
19. `process_assets_pass`
20. `after_seal_pass`
21. Unfreeze module static cache in production
