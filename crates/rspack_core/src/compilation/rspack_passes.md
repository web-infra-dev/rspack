# Compilation Passes Architecture

This document describes the modular architecture of the compilation process in rspack.

## Overview

The compilation process is organized into independent modules, each responsible for a specific phase or pass. This modular design improves code maintainability, testability, and allows for better separation of concerns.

## Module Structure

```
compilation/
├── mod.rs                        # Main Compilation struct which exposes the public API
├── run_passes.rs                 # Pass driver which calls all pass(which includes make and seal)
├── build_module_graph/           # Module graph construction
│   └── finish_module/            # Finalize module graph, async modules, dependency diagnostics
├── optimize_dependencies/        # optimizeDependencies hook + side effects artifact
├── build_chunk_graph/            # Chunk graph construction (code splitting cache + pass wrapper)
├── optimize_modules/             # optimizeModules + afterOptimizeModules hooks
├── optimize_chunks/              # optimizeChunks hook
├── optimize_tree/                # optimizeTree hook
├── optimize_chunk_modules        # optimizeChunkModules hook
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

## Pass Order

`run_passes` orchestrates passes after `CompilationHooks::seal` in this order:

1. `optimize_dependencies_pass`
2. `build_chunk_graph_pass` → `optimize_modules_pass` → `optimize_chunks_pass`
3. `optimize_tree_pass` → `optimize_chunk_modules_pass`
4. `module_ids_pass` → `chunk_ids_pass` → `assign_runtime_ids`
5. `optimize_code_generation_pass`
6. `create_module_hashes_pass`
7. `code_generation_pass`
8. `runtime_requirements_pass`
9. `create_hash_pass` (also runs runtime module code generation)
10. `create_module_assets_pass`
11. `create_chunk_assets_pass`
12. `process_assets_pass`
13. `after_seal_pass`
