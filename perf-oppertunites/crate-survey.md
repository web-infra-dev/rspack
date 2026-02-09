# Crate-by-Crate Survey

This survey walks every crate under `crates/` and records its role plus a
performance‑relevant note. The intent is to provide a map of where to look when
profiling suggests a hotspot.

## Core Engine & Infrastructure

| Crate | Purpose | Perf notes |
| --- | --- | --- |
| `rspack` | Top-level crate entrypoint. | Keep build surfaces thin; avoid extra allocation glue. |
| `rspack_core` | Core compilation engine (module graph, chunking, codegen). | Primary hotspot domain; focus on allocation, graph updates, codegen jobs. |
| `rspack_workspace` | Workspace utilities shared across crates. | Ensure utilities avoid per-module allocations. |
| `rspack_tasks` | Task scheduling utilities. | Favor batching/parallelism while minimizing task overhead. |
| `rspack_util` | Shared helpers. | Hot helpers should avoid string allocations; prefer `Cow`. |
| `rspack_collections` | Custom data structures. | Potential target for cache-friendly layouts. |
| `rspack_ids` | ID generation/management. | Avoid hashing churn; cache derived IDs. |
| `rspack_hash` | Hashing utilities. | Hashing is pervasive; use incremental hashes/cached digests. |
| `rspack_regex` | Regex helpers. | Regex compilation and matching can be cached or precompiled. |
| `rspack_paths` | Path utilities and types. | Reduce path conversions and UTF-8 validation in hot loops. |
| `rspack_location` | Location/diagnostic metadata. | Ensure location tracking is off in release unless needed. |
| `rspack_error` | Error types & formatting. | Avoid formatting heavy strings in success paths. |
| `rspack_fs` | File system abstraction. | Hot path IO; consider batching and caching stats reads. |
| `rspack_futures` | Async helpers. | Scope spawns should minimize per-job allocations. |
| `rspack_hook` | Hook system implementation. | Hook dispatch should short-circuit when no taps. |
| `rspack_tracing` | Tracing integrations. | Ensure tracing is disabled or low-cost in release. |
| `rspack_tracing_perfetto` | Perfetto tracing backend. | Use only for profiling; keep runtime overhead gated. |
| `rspack_allocator` | mimalloc integration. | Allocation profiling indicates heavy use; tune arenas. |
| `rspack_watcher` | File watching. | Avoid unnecessary stat storms; debounce. |
| `rspack_storage` | Persistent cache storage. | Optimize serialization, compression, and IO batching. |
| `rspack_cacheable` | Cacheable serialization framework. | Ensure serializers avoid intermediate allocations. |
| `rspack_cacheable_macros` | Procedural macros for cacheable. | Keep generated code minimal to reduce runtime overhead. |
| `rspack_cacheable_test` | Cacheable tests. | Not performance sensitive; keep tests isolated. |
| `rspack_tools` | Debug/testing toolkit. | Off hot path; avoid accidental inclusion in runtime builds. |
| `rspack_browser` | Browser/WASM build support. | Keep compatibility shims small; avoid extra indirection. |
| `rspack_browserslist` | Browserslist handling. | Cache parsed config and avoid repeated parsing. |

## Bindings & API Surface

| Crate | Purpose | Perf notes |
| --- | --- | --- |
| `node_binding` (`rspack_node`) | Node.js binding entrypoint. | Avoid marshaling overhead; minimize crossing the NAPI boundary. |
| `rspack_binding_api` | Shared binding API for JS/Rust. | Batch JS↔Rust calls to reduce overhead. |
| `rspack_binding_build` | Build script for bindings. | Not runtime sensitive. |
| `rspack_binding_builder` | Binding builder helpers. | Not runtime sensitive. |
| `rspack_binding_builder_macros` | Builder macros. | Generated code should avoid heavy allocations. |
| `rspack_binding_builder_testing` | Binding builder tests. | Not runtime sensitive. |
| `rspack_napi` | NAPI wrappers. | Keep type conversions lean; avoid cloning large buffers. |
| `rspack_napi_macros` | NAPI macros. | Prefer zero-copy buffers where possible. |

## Compilers & Loaders

| Crate | Purpose | Perf notes |
| --- | --- | --- |
| `rspack_javascript_compiler` | SWC-based JS parser/transform. | High-cost path; reuse AST allocations and cache configs. |
| `rspack_loader_runner` | Loader pipeline runner. | Loader state machine is hot; avoid repeated string conversions. |
| `rspack_loader_swc` | Built-in SWC loader. | Reduce config churn; reuse SWC instances if possible. |
| `rspack_loader_lightningcss` | LightningCSS loader. | Keep CSS parsing cached for unchanged assets. |
| `rspack_loader_react_refresh` | React refresh loader. | Gate for dev only; ensure no prod overhead. |
| `rspack_loader_preact_refresh` | Preact refresh loader. | Gate for dev only. |
| `rspack_loader_testing` | Loader tests. | Not runtime sensitive. |
| `swc_plugin_import` | SWC plugin for import transforms. | Cache plugin state and avoid re-parsing options. |
| `swc_plugin_ts_collector` | SWC plugin for TS metadata. | Avoid heavy AST traversals when not needed. |

## Plugins

| Crate | Purpose | Perf notes |
| --- | --- | --- |
| `rspack_plugin_asset` | Asset module handling. | Avoid re-hashing asset content; reuse buffers. |
| `rspack_plugin_banner` | Banner injection. | Off hot path; short-circuit when disabled. |
| `rspack_plugin_case_sensitive` | Case sensitivity checks. | File system checks can be cached per path. |
| `rspack_plugin_circular_dependencies` | Circular dep detection. | Graph traversal can be expensive; run only when enabled. |
| `rspack_plugin_copy` | Copy files to output. | Batch file IO; avoid per-file stat overhead. |
| `rspack_plugin_css` | CSS processing plugin. | CSS parsing/minifying costs; cache results for unchanged inputs. |
| `rspack_plugin_css_chunking` | CSS chunking. | Chunk graph operations; optimize chunk grouping. |
| `rspack_plugin_devtool` | Source map/devtool logic. | Source map generation is expensive; ensure gating. |
| `rspack_plugin_dll` | DLL plugin support. | IO heavy; cache manifest parsing. |
| `rspack_plugin_dynamic_entry` | Dynamic entry handling. | Avoid repeated resolution of dynamic entrypoints. |
| `rspack_plugin_entry` | Entry plugin. | Ensure entry creation does not clone large structures. |
| `rspack_plugin_ensure_chunk_conditions` | Chunk condition enforcement. | Avoid repeated scans; cache decisions. |
| `rspack_plugin_esm_library` | ESM library output support. | Keep output formatting light. |
| `rspack_plugin_externals` | Externalization of modules. | Avoid repeated resolution for externals. |
| `rspack_plugin_extract_css` | Extract CSS assets. | Combine CSS ops; avoid string concatenation in loops. |
| `rspack_plugin_hmr` | HMR runtime. | Dev-only; ensure gating. |
| `rspack_plugin_html` | HTML generation. | Template rendering can be cached. |
| `rspack_plugin_ignore` | Ignore rules. | Fast path for ignore checks (hash set). |
| `rspack_plugin_javascript` | JS pipeline plugin. | Large hot path; keep parsing/codegen cacheable. |
| `rspack_plugin_json` | JSON module support. | Avoid string conversions when possible. |
| `rspack_plugin_lazy_compilation` | Lazy compilation. | Ensure low overhead for normal builds. |
| `rspack_plugin_library` | Library output plugin. | Output formatting; keep operations linear. |
| `rspack_plugin_lightning_css_minimizer` | CSS minifier. | Gate to production builds; tune concurrency. |
| `rspack_plugin_limit_chunk_count` | Limit chunk count. | Graph traversal; consider early exits. |
| `rspack_plugin_merge_duplicate_chunks` | Merge duplicate chunks. | Hashing/comparison heavy; use fingerprints. |
| `rspack_plugin_mf` | Module federation. | Large runtime output; cache manifest work. |
| `rspack_plugin_module_info_header` | Module info header injection. | Avoid string formatting per module in hot builds. |
| `rspack_plugin_module_replacement` | Module replacement. | Cache replacement decisions. |
| `rspack_plugin_no_emit_on_errors` | Emission gating. | Quick checks only. |
| `rspack_plugin_progress` | Progress reporting. | Avoid frequent string allocations. |
| `rspack_plugin_real_content_hash` | Real content hash. | Hashing cost; reuse digests. |
| `rspack_plugin_remove_duplicate_modules` | Deduplicate modules. | Graph scans; use fast set ops. |
| `rspack_plugin_remove_empty_chunks` | Remove empty chunks. | Avoid full graph passes when no changes. |
| `rspack_plugin_rsdoctor` | Rsdoctor integration. | Profiling-only; ensure gating. |
| `rspack_plugin_rsc` | React Server Components. | High-cost transforms; cache on unchanged sources. |
| `rspack_plugin_rslib` | Rslib integration. | Off hot path for normal bundling. |
| `rspack_plugin_rstest` | Rstest integration. | Off hot path. |
| `rspack_plugin_runtime` | Runtime modules. | Runtime generation can be cached; avoid repeated template rendering. |
| `rspack_plugin_runtime_chunk` | Runtime chunk split. | Chunk graph operations; consider incremental updates. |
| `rspack_plugin_schemes` | Scheme handling. | IO + URL parsing; cache scheme dispatch. |
| `rspack_plugin_size_limits` | Size limits warnings. | Keep checks lightweight. |
| `rspack_plugin_split_chunks` | SplitChunks implementation. | Heavy graph cost; optimize group calculation and reuse results. |
| `rspack_plugin_sri` | Subresource integrity. | Hashing over assets; consider parallel hashing. |
| `rspack_plugin_swc_js_minimizer` | SWC JS minifier. | Gate to prod; parallelize minification. |
| `rspack_plugin_wasm` | WASM support. | Parsing/emit cost; cache module analysis. |
| `rspack_plugin_web_worker_template` | Web worker templates. | Template rendering can be cached. |
| `rspack_plugin_worker` | Worker plugin. | Module graph additions; cache resolved worker entries. |
