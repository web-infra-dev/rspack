# rspack_napi

## Role
NAPI integration and wrappers for JS↔Rust interop.

## Perf opportunities
- Batch NAPI calls to reduce crossing overhead.
- Use zero-copy buffers for sources/assets.
- Avoid cloning large objects when passing to JS.

## Code pointers
- `crates/rspack_napi/**`
