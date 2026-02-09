# rspack_tracing_perfetto

## Role
Perfetto tracing backend for detailed profiling.

## Perf opportunities
- Ensure tracing is disabled in production builds by default.
- Avoid large trace payloads or per-entity events in hot loops.
- Use sampling strategies instead of full event emission.

## Code pointers
- `crates/rspack_tracing_perfetto/**`
