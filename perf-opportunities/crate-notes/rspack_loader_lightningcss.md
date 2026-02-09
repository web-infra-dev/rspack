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

## Key functions/structs to inspect
- Loader plugin wiring in `plugin.rs`.
- Config normalization in `config.rs`.
- `process`/transform flow in `lib.rs`.

## Suggested experiments
- Profile CSS-heavy builds and measure LightningCSS parsing time.
- Compare cache hit rates for unchanged CSS modules.

## Code pointers
- `crates/rspack_loader_lightningcss/Cargo.toml`
- `crates/rspack_loader_lightningcss/src/lib.rs`
- `crates/rspack_loader_lightningcss/src/config.rs`
- `crates/rspack_loader_lightningcss/src/plugin.rs`
- `crates/rspack_loader_lightningcss/**`
