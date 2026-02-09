# rspack_plugin_externals

## Role
Externalization of modules based on configuration.

## Profiling relevance
- Not visible in react-10k; hot when many externals are configured.
- Costs scale with resolution checks per module.

## Perf opportunities
- Cache external resolution decisions by request/context.
- Avoid resolver work when externals match early.
- Batch external checks for repeated specifiers.

## Suggested experiments
- Profile builds with large externals lists to measure resolver calls.
- Compare cached vs uncached external decision performance.

## Code pointers
- `crates/rspack_plugin_externals/Cargo.toml`
- `crates/rspack_plugin_externals/src/lib.rs`
- `crates/rspack_plugin_externals/**`
