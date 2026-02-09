# rspack_binding_api

## Role
Shared binding API surface for JS↔Rust integration.

## Perf opportunities
- Batch binding calls to reduce NAPI overhead.
- Prefer zero-copy buffers for assets and sources.
- Avoid deep cloning when transferring large objects.

## Code pointers
- `crates/rspack_binding_api/**`
