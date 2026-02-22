# rspack_plugin_library

## Role
Library output configuration and wrapper generation.

## Profiling relevance
- Not visible in react-10k; hot for library builds with many entrypoints.
- Costs scale with wrapper generation and export mapping.

## Perf opportunities
- Cache library wrapper templates by target/library type.
- Avoid string concatenation in per-module wrappers.
- Skip library formatting when output is not a library target.

## Key functions/structs to inspect
- `AssignLibraryPlugin` / `AmdLibraryPlugin` hooks (assign_library_plugin.rs, amd_library_plugin.rs).
- `UmdLibraryPlugin` output formatting (umd_library_plugin.rs).
- Helper utilities in `utils.rs`.

## Suggested experiments
- Profile library builds to measure wrapper generation time.
- Compare cached templates across incremental builds.

## Code pointers
- `crates/rspack_plugin_library/Cargo.toml`
- `crates/rspack_plugin_library/src/lib.rs`
- `crates/rspack_plugin_library/src/amd_library_plugin.rs`
- `crates/rspack_plugin_library/src/modern_module_library_plugin.rs`
- `crates/rspack_plugin_library/src/umd_library_plugin.rs`
- `crates/rspack_plugin_library/**`
