# rspack_plugin_lightning_css_minimizer

## Role
CSS minification using LightningCSS.

## Profiling relevance
- Not visible in react-10k perf samples; hot in production builds with CSS minification.
- Costs scale with CSS size and number of CSS modules.

## Perf opportunities
- Cache minifier configuration and reuse parser state.
- Parallelize minification with bounded concurrency.
- Avoid re-minifying unchanged CSS modules via content hash caching.
- Single-file crate: concentrate profiling on `src/lib.rs` hook implementations.

## Suggested experiments
- Profile CSS-heavy builds with minification enabled/disabled.
- Measure cache hit rate for unchanged CSS assets.

## Code pointers
- `crates/rspack_plugin_lightning_css_minimizer/Cargo.toml`
- `crates/rspack_plugin_lightning_css_minimizer/src/lib.rs`
- `crates/rspack_plugin_lightning_css_minimizer/**`
