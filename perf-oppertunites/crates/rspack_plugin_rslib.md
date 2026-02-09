# rspack_plugin_rslib

## Role
Rslib integration for library builds.

## Profiling relevance
- Not visible in react-10k; relevant when building libraries.
- Costs scale with wrapper generation and manifest output.

## Perf opportunities
- Avoid extra manifest generation when library mode is not active.
- Cache library output templates by configuration.
- Reduce per-module string formatting in library wrappers.

## Suggested experiments
- Profile library builds with large module graphs.
- Measure impact of cached wrapper templates.

## Code pointers
- `crates/rspack_plugin_rslib/Cargo.toml`
- `crates/rspack_plugin_rslib/src/lib.rs`
- `crates/rspack_plugin_rslib/**`
