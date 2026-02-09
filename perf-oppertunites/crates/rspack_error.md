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
