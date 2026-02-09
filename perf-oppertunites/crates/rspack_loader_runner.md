# rspack_loader_runner

## Role
Loader execution pipeline and state machine.

## Perf opportunities
- Reuse loader context buffers and dependency sets.
- Avoid lossy string conversions on large sources.
- Batch loader execution for small modules to reduce overhead.

## Code pointers
- `crates/rspack_loader_runner/**`
