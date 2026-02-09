# rspack_loader_runner

## Role
Loader execution pipeline and state machine.

## Profiling relevance
- Related to `from_utf8_lossy` hotspot via loader content conversions.
- Costs scale with number of loaders and modules.

## Perf opportunities
- Reuse loader context buffers and dependency sets.
- Avoid lossy string conversions on large sources.
- Batch loader execution for small modules to reduce overhead.

## Suggested experiments
- Measure loader pipeline time with varying loader counts.
- Compare zero-copy content handling vs lossy conversions.

## Code pointers
- `crates/rspack_loader_runner/Cargo.toml`
- `crates/rspack_loader_runner/src/lib.rs`
- `crates/rspack_loader_runner/src/runner.rs`
- `crates/rspack_loader_runner/src/content.rs`
- `crates/rspack_loader_runner/src/context.rs`
- `crates/rspack_loader_runner/**`
