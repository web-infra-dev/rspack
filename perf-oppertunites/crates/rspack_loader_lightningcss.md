# rspack_loader_lightningcss

## Role
LightningCSS loader for CSS parsing/transform.

## Perf opportunities
- Cache LightningCSS parser configuration per module type.
- Avoid re-parsing unchanged CSS modules.
- Reuse buffers for CSS output to reduce allocations.

## Code pointers
- `crates/rspack_loader_lightningcss/**`
