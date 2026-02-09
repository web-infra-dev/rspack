# rspack_plugin_lightning_css_minimizer

## Role
CSS minification using LightningCSS.

## Perf opportunities
- Cache minifier configuration and reuse parser state.
- Parallelize minification with bounded concurrency.
- Avoid re-minifying unchanged CSS modules via content hash caching.

## Code pointers
- `crates/rspack_plugin_lightning_css_minimizer/**`
