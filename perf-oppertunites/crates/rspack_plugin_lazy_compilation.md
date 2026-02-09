# rspack_plugin_lazy_compilation

## Role
Lazy compilation support for deferred module building.

## Profiling relevance
- Not visible in react-10k; hot when lazy compilation is enabled.
- Costs scale with number of lazy entry points.

## Perf opportunities
- Ensure lazy compilation checks are low overhead when disabled.
- Cache module eligibility results to avoid repeated checks.
- Avoid runtime template regeneration for identical lazy entries.

## Key functions/structs to inspect
- Backend/driver logic in `backend.rs`.
- Module stubs in `module.rs`.
- Plugin hooks in `plugin.rs`.

## Suggested experiments
- Profile lazy compilation builds to measure eligibility checks.
- Compare cached vs uncached lazy entry handling.

## Code pointers
- `crates/rspack_plugin_lazy_compilation/Cargo.toml`
- `crates/rspack_plugin_lazy_compilation/src/lib.rs`
- `crates/rspack_plugin_lazy_compilation/src/backend.rs`
- `crates/rspack_plugin_lazy_compilation/src/dependency.rs`
- `crates/rspack_plugin_lazy_compilation/src/plugin.rs`
- `crates/rspack_plugin_lazy_compilation/**`
