# rspack_tracing

## Role
Tracing integration utilities used across compilation phases.

## Profiling relevance
- Present only when tracing is enabled.
- Can add overhead in high‑volume loops if spans are created unconditionally.

## Perf opportunities
- Gate span creation behind feature flags or log level checks.
- Avoid building trace payload strings in hot paths.
- Batch trace events or reduce event cardinality for high-volume loops.

## Key functions/structs to inspect
- `Tracer::setup`, `Tracer::sync_trace`, `Tracer::teardown` (tracer.rs).
- `TraceEvent` struct (tracer.rs) and payload size per event.
- `stdout`/`perfetto` layer implementations (stdout.rs, perfetto.rs).

## Suggested experiments
- Compare build times with tracing enabled vs disabled on large workloads.
- Measure overhead of hot spans in module graph and loader loops.

## Code pointers
- `crates/rspack_tracing/Cargo.toml`
- `crates/rspack_tracing/src/lib.rs`
- `crates/rspack_tracing/src/perfetto.rs`
- `crates/rspack_tracing/src/stdout.rs`
- `crates/rspack_tracing/src/tracer.rs`
- `crates/rspack_tracing/**`
