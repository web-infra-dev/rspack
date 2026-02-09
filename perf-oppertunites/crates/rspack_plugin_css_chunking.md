# rspack_plugin_css_chunking

## Role
CSS chunking optimization for splitting CSS output.

## Perf opportunities
- Cache CSS chunk group computations.
- Avoid full graph scans when CSS modules are unchanged.
- Batch CSS chunk rendering to reduce IO overhead.

## Code pointers
- `crates/rspack_plugin_css_chunking/**`
