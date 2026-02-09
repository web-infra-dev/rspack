# rspack_loader_lightningcss

## Role
LightningCSS loader for CSS parsing/transform.

## Profiling relevance
- Not visible in react-10k; hot for CSS-heavy projects.
- Costs scale with CSS size and transform complexity.

## Perf opportunities
- Cache LightningCSS parser configuration per module type.
- Avoid re-parsing unchanged CSS modules.
- Reuse buffers for CSS output to reduce allocations.

## Suggested experiments
- Profile CSS-heavy builds and measure LightningCSS parsing time.
- Compare cache hit rates for unchanged CSS modules.

## Code pointers
- `crates/rspack_loader_lightningcss/Cargo.toml`
- `crates/rspack_loader_lightningcss/src/lib.rs`
- `crates/rspack_loader_lightningcss/**`
