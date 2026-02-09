# rspack_plugin_library

## Role
Library output configuration and wrapper generation.

## Profiling relevance
- Not visible in react-10k; hot for library builds with many entrypoints.
- Costs scale with wrapper generation and export mapping.

## Perf opportunities
- Cache library wrapper templates by target/library type.
- Avoid string concatenation in per-module wrappers.
- Skip library formatting when output is not a library target.

## Suggested experiments
- Profile library builds to measure wrapper generation time.
- Compare cached templates across incremental builds.

## Code pointers
- `crates/rspack_plugin_library/Cargo.toml`
- `crates/rspack_plugin_library/src/lib.rs`
- `crates/rspack_plugin_library/**`
