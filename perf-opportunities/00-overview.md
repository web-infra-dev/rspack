# Rspack Performance Opportunities — Overview

## Purpose

This directory contains a **per-crate performance analysis** of every significant Rust crate in the Rspack bundler. Each document identifies concrete optimization opportunities discovered through manual source-code inspection, architectural analysis, and profiling trace review.

## Reference Workload: react-10k

All analysis references the **react-10k** benchmark from [rstackjs/build-tools-performance](https://github.com/rstackjs/build-tools-performance/tree/main/cases/react-10k). This workload simulates a large React application with ~10,000 components, representing a realistic enterprise-scale project.

At this scale:
- The **module graph** contains 10,000+ modules with deep import chains
- **Code splitting** decisions affect thousands of modules simultaneously
- **Hashing** involves 10,000+ hash computations per compilation pass
- **Code generation** runs for each module × each runtime configuration
- **Tree shaking** must analyze exports/imports across all 10,000+ modules
- **Module concatenation** evaluates every ESM module for scope hoisting eligibility

## Rspack Compilation Pipeline

The compilation follows a sequential pass-based architecture (from `crates/rspack_core/src/compilation/run_passes.rs`):

```
Phase 1 — Build Module Graph (make)
  BuildModuleGraphPhasePass         ← Factorize → Build → ProcessDeps (parallel task loop)

Phase 2 — Seal & Optimize
  SealPass                          ← Freeze module graph
  OptimizeDependenciesPass          ← Side effects, flag exports/usage
  BuildChunkGraphPass               ← Code splitting (CodeSplitter BFS)
  OptimizeModulesPass               ← Module-level optimizations
  OptimizeChunksPass                ← Split chunks plugin
  OptimizeTreePass                  ← Tree-level optimizations
  OptimizeChunkModulesPass          ← Module concatenation
  ModuleIdsPass                     ← Assign module IDs
  ChunkIdsPass                      ← Assign chunk IDs
  AssignRuntimeIdsPass              ← Assign runtime IDs

Phase 3 — Code Generation & Hashing
  OptimizeCodeGenerationPass        ← Mangle exports
  CreateModuleHashesPass            ← Hash each module × runtime
  CodeGenerationPass                ← Generate code (parallel)
  RuntimeRequirementsPass           ← Compute runtime requirements
  CreateHashPass                    ← Chunk hashes, full hash, runtime hash
  CreateModuleAssetsPass            ← Module asset emission
  CreateChunkAssetsPass             ← Render chunk manifests (parallel)

Phase 4 — Finalize
  ProcessAssetsPass                 ← Post-processing (minification, etc.)
  AfterProcessAssetsPass
  AfterSealPass
```

## Crate Inventory (by lines of Rust code)

| Crate | Lines | Files | Role |
|-------|-------|-------|------|
| rspack_core | 55,113 | 236 | Central compilation engine |
| rspack_plugin_javascript | 39,435 | 151 | JS parsing, tree shaking, codegen |
| rspack_binding_api | 23,251 | 125 | NAPI JS↔Rust bindings |
| rspack_plugin_mf | 7,630 | 45 | Module federation |
| rspack_plugin_runtime | 7,480 | 63 | Runtime code generation |
| rspack_storage | 6,414 | 28 | Persistent cache |
| rspack_plugin_css | 3,316 | 14 | CSS handling |
| rspack_plugin_split_chunks | 2,705 | 13 | Code splitting |
| rspack_cacheable | 2,736 | 36 | Serialization |
| rspack_util | 2,447 | 24 | Utilities |
| rspack_loader_swc | 4,399 | 11 | SWC loader |
| rspack_loader_runner | 1,853 | 7 | Loader pipeline |
| rspack_hash | ~250 | 1 | Hashing |
| rspack_futures | ~200 | 2 | Async parallelism |
| rspack_collections | ~400 | 3 | Specialized collections |
| rspack_allocator | ~15 | 1 | Global allocator |

## Document Index

### Per-Crate Analysis (1 file per crate)

| # | File | Crate | Key Opportunities |
|---|------|-------|-------------------|
| 01 | `01-rspack-core.md` | rspack_core | Module graph allocation, code splitter BigUint, sequential passes, hashing |
| 02 | `02-rspack-plugin-javascript.md` | rspack_plugin_javascript | Parsing, AST walking, tree shaking, code generation |
| 03 | `03-rspack-plugin-split-chunks.md` | rspack_plugin_split_chunks | Module group computation, chunk indexing |
| 04 | `04-rspack-hash.md` | rspack_hash | Hash function choice, digest allocation |
| 05 | `05-rspack-futures.md` | rspack_futures | Async scope overhead, task spawning |
| 06 | `06-rspack-loader-swc.md` | rspack_loader_swc | SWC transform pipeline, source map handling |
| 07 | `07-rspack-loader-runner.md` | rspack_loader_runner | Loader chain execution |
| 08 | `08-rspack-collections.md` | rspack_collections | Identifier hashing, UkeyMap layout |
| 09 | `09-rspack-plugin-css.md` | rspack_plugin_css | CSS parsing and code generation |
| 10 | `10-rspack-allocator.md` | rspack_allocator | mimalloc configuration, memory patterns |
| 11 | `11-rspack-storage.md` | rspack_storage | Persistent cache I/O, serialization |
| 12 | `12-rspack-binding-api.md` | rspack_binding_api | NAPI overhead, JS↔Rust boundary |
| 13 | `13-rspack-plugin-runtime.md` | rspack_plugin_runtime | Runtime module generation |
| 14 | `14-rspack-resolver.md` | rspack_resolver | Module resolution caching |
| 15 | `15-rspack-plugin-mf.md` | rspack_plugin_mf | Module federation overhead |
| 16 | `16-rspack-cacheable.md` | rspack_cacheable | Serialization performance |
| 17 | `17-remaining-crates.md` | rspack_error, rspack_hook, rspack_ids, rspack_regex, rspack_tracing, rspack_util, etc. | Smaller crates |

### Profiling & Deep Dives

| # | File | Focus | Key Finding |
|---|------|-------|-------------|
| 18 | `18-prioritized-recommendations.md` | All crates | 26 recommendations ranked in 5 tiers with implementation roadmap |
| 19 | `19-profiling-analysis.md` | Real traces | Actual build timing at 200/500 modules with scaling projections |
| 20 | `20-deep-dive-sideeffects-bottleneck.md` | SideEffectsFlagPlugin | **#1 bottleneck**: O(n²) scaling in `update connections` |
| 21 | `21-deep-dive-codesplitter-biguint.md` | CodeSplitter | Line-by-line BigUint hot path analysis, FixedBitSet proposal |
| 22 | `22-deep-dive-task-loop-main-thread.md` | Task Loop | 950K sequential HashMap ops, concurrent graph proposal |
| 23 | `23-deep-dive-exports-info-system.md` | ExportsInfo | Root cause of O(n²): per-task fresh caches in side effects |
| 24 | `24-deep-dive-parsing-scanning.md` | Parsing Pipeline | 7-pass pipeline, 140M node visits at 10K modules |
| 25 | `25-deep-dive-incremental-compilation.md` | Incremental | 5 disabled passes in production, BUILD_CHUNK_GRAPH disabled |

## Methodology

1. **Manual source code inspection**: Every significant `.rs` file in each crate was read and analyzed
2. **Algorithmic complexity analysis**: Data structure choices, loop patterns, allocation patterns
3. **Concurrency analysis**: Lock contention, parallel utilization, sequential bottlenecks
4. **Memory analysis**: Allocation hot spots, clone patterns, data structure overhead
5. **Profiling trace analysis**: Using `rspack_tracing` Perfetto integration and `#[instrument]` annotations
6. **Benchmark context**: All findings evaluated against the react-10k workload profile
