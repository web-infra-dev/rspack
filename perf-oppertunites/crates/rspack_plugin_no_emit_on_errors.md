# rspack_plugin_no_emit_on_errors

## Role
Suppress asset emission when compilation errors are present.

## Profiling relevance
- Not in hot path for successful builds.
- Should be near-zero overhead when no errors occur.

## Perf opportunities
- Keep checks lightweight; avoid scanning full diagnostics.
- Reuse cached error flags from earlier passes.
- Avoid allocations when no errors are present.
- Single-file crate: concentrate profiling on `src/lib.rs` hook implementations.

## Suggested experiments
- Measure error detection overhead on error-free builds.
- Validate short-circuit behavior when errors are present.

## Code pointers
- `crates/rspack_plugin_no_emit_on_errors/Cargo.toml`
- `crates/rspack_plugin_no_emit_on_errors/src/lib.rs`
- `crates/rspack_plugin_no_emit_on_errors/**`
