# rspack_plugin_ignore

## Role
Ignore plugin for excluding modules based on patterns.

## Profiling relevance
- Not visible in react-10k; can be hot when many ignore rules are configured.
- Costs scale with pattern matching frequency.

## Perf opportunities
- Precompile ignore patterns and reuse across builds.
- Short-circuit resolution when ignore hits early.
- Avoid allocating new regex objects per module.

## Suggested experiments
- Profile builds with heavy ignore rules to measure matcher overhead.
- Compare precompiled regex vs per-module compilation.

## Code pointers
- `crates/rspack_plugin_ignore/Cargo.toml`
- `crates/rspack_plugin_ignore/src/lib.rs`
- `crates/rspack_plugin_ignore/**`
