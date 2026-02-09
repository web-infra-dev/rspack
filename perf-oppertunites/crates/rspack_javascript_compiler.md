# rspack_javascript_compiler

## Role
SWC-backed JavaScript/TypeScript parsing and transformation.

## Perf opportunities
- Cache SWC configs and reuse AST arenas between modules.
- Avoid repeated UTF‑8 validation of source buffers.
- Use parallel parsing with bounded concurrency to prevent oversubscription.

## Code pointers
- `crates/rspack_javascript_compiler/**`
