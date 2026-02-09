# swc_plugin_import

## Role
SWC plugin that implements `babel-plugin-import` behavior in Rust.

## Profiling relevance
- Not directly observed in react-10k perf samples; depends on use of import rewrites.
- Hot when large codebases rely on tree‑shaken import transforms.

## Perf opportunities
- Cache resolved import mappings to avoid recomputing per module.
- Avoid repeated string allocations when generating new import paths.
- Short-circuit when module has no matching import patterns.

## Suggested experiments
- Profile a case with heavy `babel-plugin-import` usage and compare cached vs. uncached behavior.
- Measure allocations during path rewriting.

## Code pointers
- `crates/swc_plugin_import/Cargo.toml`
- `crates/swc_plugin_import/src/lib.rs`
- `crates/swc_plugin_import/**`
