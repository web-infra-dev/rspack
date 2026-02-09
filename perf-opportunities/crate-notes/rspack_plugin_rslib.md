# rspack_plugin_rslib

## Role
Rslib integration for library builds.

## Profiling relevance
- Not visible in react-10k; relevant when building libraries.
- Costs scale with wrapper generation and manifest output.

## Perf opportunities
- Avoid extra manifest generation when library mode is not active.
- Cache library output templates by configuration.
- Reduce per-module string formatting in library wrappers.

## Key functions/structs to inspect
- `RslibPlugin` hook registrations (plugin.rs).
- Parser plugin hooks in `parser_plugin.rs`.
- Asset helpers in `asset.rs` and dependency handling in `import_dependency.rs`.

## Suggested experiments
- Profile library builds with large module graphs.
- Measure impact of cached wrapper templates.

## Code pointers
- `crates/rspack_plugin_rslib/Cargo.toml`
- `crates/rspack_plugin_rslib/src/lib.rs`
- `crates/rspack_plugin_rslib/src/plugin.rs`
- `crates/rspack_plugin_rslib/src/parser_plugin.rs`
- `crates/rspack_plugin_rslib/src/asset.rs`
- `crates/rspack_plugin_rslib/**`
