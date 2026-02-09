# rspack_loader_swc

## Role
Built-in SWC loader for JS/TS transformations.

## Perf opportunities
- Cache normalized SWC loader options per module type.
- Avoid re-instantiating SWC components per module.
- Reuse buffers for loader outputs to reduce allocations.

## Code pointers
- `crates/rspack_loader_swc/**`
