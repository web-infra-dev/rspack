# rspack_plugin_web_worker_template

## Role
Template generation for web worker runtime.

## Profiling relevance
- Not visible in react-10k; hot when web worker templates are emitted.
- Costs scale with number of worker chunks.

## Perf opportunities
- Cache rendered templates for identical options/runtime.
- Avoid repeated string concatenation by preallocating buffers.
- Short-circuit when worker templates are not used.

## Suggested experiments
- Measure template generation time with many worker entries.
- Compare reuse of cached templates across rebuilds.

## Code pointers
- `crates/rspack_plugin_web_worker_template/Cargo.toml`
- `crates/rspack_plugin_web_worker_template/src/lib.rs`
- `crates/rspack_plugin_web_worker_template/**`
