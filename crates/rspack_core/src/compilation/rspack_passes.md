# Compilation Passes Architecture

This document describes the modular architecture of the compilation process in rspack.

## Overview

The compilation process is organized into independent modules, each responsible for a specific phase or pass. This modular design improves code maintainability, testability, and allows for better separation of concerns.

## Module Structure

```
compilation/
├── mod.rs                        # Main Compilation struct which exposes the public API
├── run_passes.rs                 # Pass driver invoked from Compiler that runs make + seal passes
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

## Pass Entry

- `Compiler::compile` builds `CompilationParams`, fires `thisCompilation` then `compilation` compiler hooks (binding safety for JS), and delegates to `Compilation::run_passes`.
- `Compilation::run_passes` performs the make and seal stages using the order below.

## Pass Order (Compilation::run_passes)

`run_passes` orchestrates the full pipeline (make + seal) in this order:

1. Make: `make` hook → `build_module_graph` → `finish_make` hook → `finish_build_module_graph`
2. Collect make diagnostics (`collect_build_module_graph_effects`)
3. Incremental checkpoint (`module_graph`), freeze module static cache in production
4. Seal kickoff: `CompilationHooks::seal`
5. `optimize_dependencies_pass`
6. `build_chunk_graph_pass` → `optimize_modules_pass` → `optimize_chunks_pass`
7. `optimize_tree_pass` → `optimize_chunk_modules_pass`
8. `module_ids_pass` → `chunk_ids_pass` → `assign_runtime_ids`
9. `optimize_code_generation_pass`
10. `create_module_hashes_pass`
11. `code_generation_pass`
12. `runtime_requirements_pass`
13. `create_hash_pass` (also runs runtime module code generation)
14. `create_module_assets_pass`
15. `create_chunk_assets_pass`
16. `process_assets_pass`
17. `after_seal_pass`
18. Unfreeze module static cache in production
