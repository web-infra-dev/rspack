# rspack_tools

## Role
Debug/testing utilities for inspecting Rspack internals.

## Profiling relevance
- Not in runtime hot path; used for diagnostics and tooling.
- Ensure no accidental invocation in production builds.

## Perf opportunities
- Ensure tooling is not accidentally invoked in production builds.
- Avoid expensive snapshot generation unless explicitly requested.
- Keep debug instrumentation gated behind feature flags.

## Suggested experiments
- Validate that tools are excluded from production builds.
- Measure overhead of any optional tooling hooks when enabled.

## Code pointers
- `crates/rspack_tools/Cargo.toml`
- `crates/rspack_tools/src/lib.rs`
- `crates/rspack_tools/**`
