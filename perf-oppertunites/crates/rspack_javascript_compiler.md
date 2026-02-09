# rspack_javascript_compiler

## Role
SWC-backed JavaScript/TypeScript parsing and transformation.

## Profiling relevance
- Related to SWC minifier and parsing hotspots in extended perf samples.
- Costs scale with module count and transform complexity.

## Perf opportunities
- Cache SWC configs and reuse AST arenas between modules.
- Avoid repeated UTF‑8 validation of source buffers.
- Use parallel parsing with bounded concurrency to prevent oversubscription.

## Suggested experiments
- Profile large JS builds with and without cached SWC configs.
- Measure AST arena reuse impact on allocation pressure.

## Code pointers
- `crates/rspack_javascript_compiler/Cargo.toml`
- `crates/rspack_javascript_compiler/src/lib.rs`
- `crates/rspack_javascript_compiler/**`
