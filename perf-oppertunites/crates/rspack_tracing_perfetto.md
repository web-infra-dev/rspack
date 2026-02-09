# rspack_tracing_perfetto

## Role
Perfetto tracing backend for detailed profiling.

## Profiling relevance
- Only active when Perfetto tracing is enabled for profiling.
- Potentially high overhead if enabled in production builds.

## Perf opportunities
- Ensure tracing is disabled in production builds by default.
- Avoid large trace payloads or per-entity events in hot loops.
- Use sampling strategies instead of full event emission.

## Suggested experiments
- Profile the impact of Perfetto tracing on a large build with different sampling rates.
- Validate that Perfetto tracing is fully gated behind flags.

## Code pointers
- `crates/rspack_tracing_perfetto/Cargo.toml`
- `crates/rspack_tracing_perfetto/src/lib.rs`
- `crates/rspack_tracing_perfetto/**`
