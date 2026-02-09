# rspack_plugin_mf

## Role
Module Federation integration (shared/remote modules, manifests).

## Perf opportunities
- Cache manifest generation and shared module analysis.
- Avoid repeated serialization of federation data to JS.
- Reduce string allocations when building federation runtime.

## Code pointers
- `crates/rspack_plugin_mf/**`
