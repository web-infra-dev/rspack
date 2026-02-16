# rspack_error

## Role
Error and diagnostic types.

## Profiling relevance
- Not in hot path for successful builds.
- Can add overhead when formatting rich diagnostics.

## Perf opportunities
- Avoid formatting heavy strings in success paths.
- Use lazy diagnostics creation when possible.
- Reduce allocations in error wrapping by reusing buffers.

## Key functions/structs to inspect
- `Diagnostic` constructors (diagnostic.rs).
- `Error::error` and conversion helpers (error.rs, convert.rs).
- `BatchErrors` aggregation (batch_error.rs).

## Suggested experiments
- Measure diagnostic formatting overhead with large error sets.
- Compare lazy vs eager error string construction.

## Code pointers
- `crates/rspack_error/Cargo.toml`
- `crates/rspack_error/src/lib.rs`
- `crates/rspack_error/src/diagnostic.rs`
- `crates/rspack_error/src/error.rs`
- `crates/rspack_error/src/displayer/mod.rs`
- `crates/rspack_error/**`
