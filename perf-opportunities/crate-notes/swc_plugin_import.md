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

## Key functions/structs to inspect
- `ImportPlugin::visit_mut_module` (lib.rs) — main rewrite loop.
- `ImportPlugin::transform` (lib.rs) — builds rewritten import paths.
- `IdentComponent::visit_ident` / `visit_ts_type_ref` (visit.rs) — reference tracking.

## Suggested experiments
- Profile a case with heavy `babel-plugin-import` usage and compare cached vs. uncached behavior.
- Measure allocations during path rewriting.

## Code pointers
- `crates/swc_plugin_import/Cargo.toml`
- `crates/swc_plugin_import/src/lib.rs`
- `crates/swc_plugin_import/src/legacy_case.rs`
- `crates/swc_plugin_import/src/template.rs`
- `crates/swc_plugin_import/src/visit.rs`
- `crates/swc_plugin_import/**`
