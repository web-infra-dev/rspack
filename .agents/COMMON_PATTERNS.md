# Common pattern map

This file points agents to real code patterns. It is not a copy-paste template
library. Before writing new code, inspect the closest existing implementation
and follow that local shape.

## Plugin work

Start with existing plugins under `crates/rspack_plugin_*/`.

- Asset processing: `crates/rspack_plugin_banner/`,
  `crates/rspack_plugin_copy/`, `crates/rspack_plugin_html/`
- Chunk optimization: `crates/rspack_plugin_split_chunks/`,
  `crates/rspack_plugin_limit_chunk_count/`
- Runtime behavior: `crates/rspack_plugin_runtime/`,
  `crates/rspack_plugin_hmr/`
- Library output: `crates/rspack_plugin_library/`,
  `crates/rspack_plugin_esm_library/`
- Diagnostics and analysis: `crates/rspack_plugin_rsdoctor/`,
  `crates/rspack_plugin_circular_dependencies/`

When adding or changing a plugin, inspect a nearby plugin with the same hook
family. Hook names and stages matter; do not infer them from memory.

## Loader work

Start with existing loaders under `crates/rspack_loader_*/`.

- SWC transform behavior: `crates/rspack_loader_swc/`
- CSS transform behavior: `crates/rspack_loader_lightningcss/`
- React and Preact refresh behavior: `crates/rspack_loader_react_refresh/`,
  `crates/rspack_loader_preact_refresh/`
- Loader execution infrastructure: `crates/rspack_loader_runner/`

Loader context types are easy to misuse. Read the current loader and runner APIs
before adding a new loader or changing options parsing.

## Compilation pass work

Start with `.agents/PASSES.md`, then inspect
`crates/rspack_core/src/compilation/`.

- Graph construction: `build_module_graph/`, `finish_module_graph/`,
  `finish_modules/`
- Chunk planning: `build_chunk_graph/`, `optimize_chunks/`,
  `optimize_chunk_modules/`
- IDs and hashes: `module_ids/`, `chunk_ids/`, `create_module_hashes/`,
  `create_hash/`
- Runtime and assets: `runtime_requirements/`, `create_module_assets/`,
  `create_chunk_assets/`, `process_assets/`, `after_process_assets/`

If a change moves work between passes, update `.agents/PASSES.md` and check
incremental artifact bindings in `.agents/ARTIFACTS.md`.

## Incremental and cache work

Read these first:

- `.agents/ARTIFACTS.md`
- `.agents/TRANSIENT_CACHE.md`
- `crates/rspack_core/src/incremental/`
- `crates/rspack_core/src/artifacts/`
- `crates/rspack_core/src/transient_cache.rs`

Use an artifact when data may be recovered across compilations. Use
`transient_cache` only for data that must not survive beyond one compilation
lifecycle.

## Public API work

Read `.agents/API_DESIGN.md`, then inspect:

- `packages/rspack/src/`
- `packages/rspack-cli/src/`
- `website/docs/en/api/`
- Compatibility tests under `tests/rspack-test/`

Public API behavior should be locked with compatibility-style tests whenever it
touches webpack-facing behavior.

## Test placement

- Rust unit tests usually live near the code under `crates/`.
- JavaScript integration cases live under `tests/rspack-test/`.
- End-to-end behavior lives under `e2e/`.
- Snapshots should follow existing case conventions.

Prefer an existing neighboring test case over inventing a new layout.
