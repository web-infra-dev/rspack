# rspack

## Role
Top-level crate entrypoint for Rspack.

## Profiling relevance
- Not directly visible in perf samples; acts as a thin wrapper.
- Overhead should be minimal and mostly initialization.

## Perf opportunities
- Keep entrypoint thin to avoid pulling extra dependencies.
- Avoid unnecessary initialization work in the common path.
- Use lazy initialization for global state.

## Suggested experiments
- Measure startup time impact of entrypoint initialization.

## Code pointers
- `crates/rspack/**`
