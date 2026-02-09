# rspack_loader_swc

## Role
Built-in SWC loader for JS/TS transformations.

## Profiling relevance
- Closely tied to SWC parsing/minification hotspots.
- Costs scale with module count and transform complexity.

## Perf opportunities
- Cache normalized SWC loader options per module type.
- Avoid re-instantiating SWC components per module.
- Reuse buffers for loader outputs to reduce allocations.

## Suggested experiments
- Measure loader option cache hit rates on incremental builds.
- Compare per-module SWC instantiation vs reused instances.

## Code pointers
- `crates/rspack_loader_swc/Cargo.toml`
- `crates/rspack_loader_swc/src/lib.rs`
- `crates/rspack_loader_swc/**`
