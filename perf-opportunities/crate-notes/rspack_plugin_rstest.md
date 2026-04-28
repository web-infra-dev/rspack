# rspack_plugin_rstest

## Role
Rstest integration for testing workflows.

## Profiling relevance
- Not in runtime hot path; only active in test workflows.
- Ensure no overhead in production builds.

## Perf opportunities
- Ensure plugin work is gated to test runs only.
- Avoid expensive file scanning when rstest is disabled.
- Reuse parsed configuration across runs.

## Key functions/structs to inspect
- `RstestPlugin::new` and `update_source` (plugin.rs).
- `nmf_parser` hook (plugin.rs) for parser plugin injection.
- Dependency templates (`import_dependency.rs`, `mock_module_id_dependency.rs`).

## Suggested experiments
- Verify rstest hooks are disabled in production mode.
- Measure config parse time with large test suites.

## Code pointers
- `crates/rspack_plugin_rstest/Cargo.toml`
- `crates/rspack_plugin_rstest/src/lib.rs`
- `crates/rspack_plugin_rstest/src/plugin.rs`
- `crates/rspack_plugin_rstest/src/parser_plugin.rs`
- `crates/rspack_plugin_rstest/src/import_dependency.rs`
- `crates/rspack_plugin_rstest/**`
