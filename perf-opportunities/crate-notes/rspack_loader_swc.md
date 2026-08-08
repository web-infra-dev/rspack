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

## Key functions/structs to inspect
- `SwcLoaderPlugin` hooks (plugin.rs).
- `transformer::transform` pipeline (transformer.rs).
- Option normalization in `options.rs`.
- TS info collection in `collect_ts_info.rs`.

## Suggested experiments
- Measure loader option cache hit rates on incremental builds.
- Compare per-module SWC instantiation vs reused instances.

## Code pointers
- `crates/rspack_loader_swc/Cargo.toml`
- `crates/rspack_loader_swc/src/lib.rs`
- `crates/rspack_loader_swc/src/options.rs`
- `crates/rspack_loader_swc/src/transformer.rs`
- `crates/rspack_loader_swc/src/collect_ts_info.rs`
- `crates/rspack_loader_swc/src/plugin.rs`
- `crates/rspack_loader_swc/**`
