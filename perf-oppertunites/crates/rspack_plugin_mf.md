# rspack_plugin_mf

## Role
Module Federation integration (shared/remote modules, manifests).

## Profiling relevance
- Not visible in react-10k; can be heavy in MF setups with many remotes.
- Manifest and runtime string generation scale with number of shared modules.

## Perf opportunities
- Cache manifest generation and shared module analysis.
- Avoid repeated serialization of federation data to JS.
- Reduce string allocations when building federation runtime.

## Suggested experiments
- Profile a federated build with many remotes to measure manifest cost.
- Measure cache hit rates for shared module analysis across rebuilds.

## Code pointers
- `crates/rspack_plugin_mf/Cargo.toml`
- `crates/rspack_plugin_mf/src/lib.rs`
- `crates/rspack_plugin_mf/src/container/mod.rs`
- `crates/rspack_plugin_mf/src/container/container_plugin.rs`
- `crates/rspack_plugin_mf/src/manifest/mod.rs`
- `crates/rspack_plugin_mf/src/sharing/mod.rs`
- `crates/rspack_plugin_mf/**`
