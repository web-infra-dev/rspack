# rspack_napi

## Role
NAPI integration and wrappers for JS↔Rust interop.

## Profiling relevance
- Not isolated in flat perf samples; relevant when JS plugins are active.
- Overhead scales with number of JS↔Rust calls and payload size.

## Perf opportunities
- Batch NAPI calls to reduce crossing overhead.
- Use zero-copy buffers for sources/assets.
- Avoid cloning large objects when passing to JS.

## Suggested experiments
- Measure JS↔Rust call counts per build and identify top hooks.
- Compare zero-copy vs cloned buffer paths.

## Code pointers
- `crates/rspack_napi/Cargo.toml`
- `crates/rspack_napi/src/lib.rs`
- `crates/rspack_napi/**`
