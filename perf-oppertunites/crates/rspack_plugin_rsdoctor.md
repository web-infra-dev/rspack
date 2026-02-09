# rspack_plugin_rsdoctor

## Role
Rsdoctor integration for profiling and diagnostics.

## Profiling relevance
- Only active when rsdoctor is enabled.
- Potentially high data volume in large projects.

## Perf opportunities
- Ensure rsdoctor hooks are disabled in normal builds.
- Avoid heavy data collection unless explicitly enabled.
- Stream diagnostic data instead of building huge in-memory structures.

## Key functions/structs to inspect
- Hook taps in `plugin.rs` (after_code_generation, optimize_chunks, etc.).
- Graph collection helpers: `collect_modules` / `collect_module_dependencies` (module_graph.rs).
- Chunk collection helpers: `collect_chunks` / `collect_chunk_modules` (chunk_graph.rs).

## Suggested experiments
- Measure rsdoctor overhead on large builds with profiling enabled.
- Validate streaming vs batch data collection impact.

## Code pointers
- `crates/rspack_plugin_rsdoctor/Cargo.toml`
- `crates/rspack_plugin_rsdoctor/src/lib.rs`
- `crates/rspack_plugin_rsdoctor/src/plugin.rs`
- `crates/rspack_plugin_rsdoctor/src/module_graph.rs`
- `crates/rspack_plugin_rsdoctor/src/chunk_graph.rs`
- `crates/rspack_plugin_rsdoctor/**`
