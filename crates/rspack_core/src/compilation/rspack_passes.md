# Compilation Passes Architecture

This document describes the modular architecture of the compilation process in rspack.

## Overview

The compilation process is organized into independent modules, each responsible for a specific phase or pass. This modular design improves code maintainability, testability, and allows for better separation of concerns.

## Module Structure

```
compilation/
├── mod.rs                      # Main Compilation struct which expose compilation api
|-- run_passes.rs               # passes driver which calls other pass
├── build_module_graph/         # Module graph construction
|-- finish_module/              # Finish Module Graph Construction collect async_modules and dependencies_diagnostics
├── optimize_dependencies/      # optimization dependencies which includes collect side_effect_optimization info
├── build_chunk_graph/          # Chunk graph construction (code splitting)
|── optimize_modules/           # which includes optimize_modules and after_optimize_modules hooks
├── optimize_chunks/            # which includes optimize_chunks hooks
├── optimize_tree/              # which includes optimize_tree hooks
├── optimize_chunk_modules      # which includes optimize_chunk_modules hooks 
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
