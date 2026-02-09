# rspack_core

## Role
Core compilation engine (module graph, chunking, codegen, hashing).

## Profiling relevance
- Multiple hotspots in module graph overlay and export prefetch.
- Allocation pressure and path/identifier handling are visible in samples.

## Perf opportunities
- Reduce allocation pressure in module graph updates and codegen jobs.
- Optimize overlay map lookups and export prefetch caching.
- Re-enable incremental chunk graph with correctness guardrails.

## Suggested experiments
- Profile module graph update stages with allocation sampling enabled.
- Measure impact of overlay fast paths and export prefetch caching.

## Code pointers
- `crates/rspack_core/Cargo.toml`
- `crates/rspack_core/src/lib.rs`
- `crates/rspack_core/src/compilation/build_module_graph/mod.rs`
- `crates/rspack_core/src/compilation/build_chunk_graph/mod.rs`
- `crates/rspack_core/src/compilation/code_generation/mod.rs`
- `crates/rspack_core/src/exports/exports_info_getter.rs`
- `crates/rspack_core/src/utils/file_counter/mod.rs`
- `crates/rspack_core/src/compilation/**`
- `crates/rspack_core/src/module_graph/**`
