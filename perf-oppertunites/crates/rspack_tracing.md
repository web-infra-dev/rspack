# rspack_tracing

## Role
Tracing integration utilities used across compilation phases.

## Perf opportunities
- Gate span creation behind feature flags or log level checks.
- Avoid building trace payload strings in hot paths.
- Batch trace events or reduce event cardinality for high-volume loops.

## Code pointers
- `crates/rspack_tracing/**`
