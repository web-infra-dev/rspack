# rspack_tools

## Role
Debug/testing utilities for inspecting Rspack internals.

## Perf opportunities
- Ensure tooling is not accidentally invoked in production builds.
- Avoid expensive snapshot generation unless explicitly requested.
- Keep debug instrumentation gated behind feature flags.

## Code pointers
- `crates/rspack_tools/**`
