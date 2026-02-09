# rspack_error

## Role
Error and diagnostic types.

## Perf opportunities
- Avoid formatting heavy strings in success paths.
- Use lazy diagnostics creation when possible.
- Reduce allocations in error wrapping by reusing buffers.

## Code pointers
- `crates/rspack_error/**`
