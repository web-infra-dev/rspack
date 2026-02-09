# rspack_plugin_css_chunking

## Role
CSS chunking optimization for splitting CSS output.

## Profiling relevance
- Not visible in react-10k; hot when many CSS chunks are produced.
- Costs scale with CSS module count and chunking strategy.

## Perf opportunities
- Cache CSS chunk group computations.
- Avoid full graph scans when CSS modules are unchanged.
- Batch CSS chunk rendering to reduce IO overhead.

## Suggested experiments
- Profile CSS-heavy builds with chunking enabled.
- Compare cached vs full CSS chunk group computation.

## Code pointers
- `crates/rspack_plugin_css_chunking/Cargo.toml`
- `crates/rspack_plugin_css_chunking/src/lib.rs`
- `crates/rspack_plugin_css_chunking/**`
