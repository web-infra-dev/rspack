# Compilation Passes Architecture

This document describes the modular architecture of the compilation process in rspack.

## Overview

The compilation process is organized into independent modules, each responsible for a specific phase or pass. This modular design improves code maintainability, testability, and allows for better separation of concerns.

## Module Structure

```
compilation/
├── mod.rs                      # Main Compilation struct and seal() orchestration
├── build_module_graph/         # Module graph construction
├── build_chunk_graph/          # Chunk graph construction (code splitting)
├── optimize/                   # Optimization hooks
├── module_ids/                 # Module ID assignment
├── chunk_ids/                  # Chunk ID assignment
├── assign_runtime_ids/         # Runtime ID assignment
├── optimize_code_generation/   # Code generation optimization
├── create_module_hashes/       # Module hash computation
├── code_generation/            # Code generation for modules
├── runtime_requirements/       # Runtime requirements processing
├── create_hash/                # Chunk hash computation
├── create_chunk_assets/        # Asset creation
└── process_assets/             # Asset processing hooks
```

## Compilation Flow

The `seal()` function orchestrates the compilation passes in the following order:

```
┌─────────────────────────────────────────────────────────────┐
│                    seal() Entry Point                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  1. optimize_dependencies()                                  │
│     - Side effects optimization                              │
│     - Module graph optimization                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  2. build_chunk_graph/                                       │
│     - Code splitting                                         │
│     - Chunk creation                                         │
│     - Module-to-chunk assignment                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  3. optimize/                                                │
│     - optimize_chunks_phase()                                │
│       • optimize_modules hook (loop)                         │
│       • after_optimize_modules hook                          │
│       • optimize_chunks hook (loop)                          │
│     - optimize_tree_phase()                                  │
│       • optimize_tree hook                                   │
│       • optimize_chunk_modules hook                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  4. module_ids/                                              │
│     - Assigns IDs to modules                                 │
│     - Uses module_ids_artifact for incremental builds        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  5. chunk_ids/                                               │
│     - Assigns IDs to chunks                                  │
│     - Uses named_chunk_ids_artifact for incremental builds   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  6. assign_runtime_ids/                                      │
│     - Assigns runtime IDs to entry chunks                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  7. optimize_code_generation/                                │
│     - Pre-code-generation optimizations                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  8. create_module_hashes/                                    │
│     - Computes hashes for modules                            │
│     - Uses cgm_hash_artifact for incremental builds          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  9. code_generation/                                         │
│     - Generates code for each module                         │
│     - Handles code generation dependencies                   │
│     - Uses code_generation_results for caching               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  10. runtime_requirements/                                   │
│      - process_modules_runtime_requirements()                │
│      - process_chunks_runtime_requirements()                 │
│      - Determines runtime code needed                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  11. create_hash/                                            │
│      - create_hash() - Computes chunk hashes                 │
│      - runtime_modules_code_generation()                     │
│      - Uses chunk_hashes_artifact for incremental builds     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  12. create_chunk_assets/                                    │
│      - create_module_assets() - Module-level assets          │
│      - create_chunk_assets() - Chunk-level assets            │
│      - Uses chunk_render_artifact for incremental builds     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  13. process_assets/                                         │
│      - process_assets() - Asset processing hooks             │
│      - after_process_assets() - Post-processing hooks        │
│      - after_seal() - Seal completion hook                   │
└─────────────────────────────────────────────────────────────┘
```

## Module Details

### build_module_graph/
Constructs the module dependency graph by:
- Processing entry points
- Resolving module dependencies
- Building the complete module graph

### build_chunk_graph/
Performs code splitting and chunk creation:
- Splits modules into chunks based on entry points and dynamic imports
- Creates chunk groups for async loading
- Manages chunk relationships

### optimize/
Runs optimization hooks in two phases:
- **optimize_chunks_phase**: Module and chunk optimization loops
- **optimize_tree_phase**: Tree-shaking and chunk module optimization

### module_ids/ & chunk_ids/
Assigns stable IDs to modules and chunks:
- Supports deterministic, named, and natural ID strategies
- Maintains artifacts for incremental compilation

### code_generation/
Generates the actual code for each module:
- Handles modules with and without code generation dependencies
- Caches results for incremental builds

### runtime_requirements/
Processes runtime requirements:
- Determines what runtime helpers are needed
- Handles module-level and chunk-level requirements
- Manages tree-level requirements for entry points

### create_hash/
Computes content hashes:
- Module hashes for cache busting
- Chunk hashes for output filenames
- Supports full hash dependencies

### create_chunk_assets/
Creates the final output assets:
- Renders chunks to source files
- Handles auxiliary assets
- Manages asset info and metadata

### process_assets/
Final asset processing:
- Runs process_assets hooks for plugins
- Handles post-processing transformations
- Completes the seal phase

## Incremental Compilation

Many passes support incremental compilation through artifacts:
- `module_ids_artifact` - Cached module IDs
- `named_chunk_ids_artifact` - Cached chunk IDs
- `cgm_hash_artifact` - Cached module hashes
- `code_generation_results` - Cached code generation
- `chunk_hashes_artifact` - Cached chunk hashes
- `chunk_render_artifact` - Cached chunk renders

The `reset_artifact_if_passes_disabled()` function handles artifact state management when incremental passes are disabled.

## Adding New Passes

To add a new pass:

1. Create a new module folder under `compilation/`
2. Export the main function taking `&mut Compilation` as the first parameter
3. Add the module declaration in `mod.rs`
4. Call the function from `seal()` at the appropriate point
5. If the pass supports incremental compilation, add artifact handling
