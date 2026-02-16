# Hotspot → Code Map

This table maps perf symbols to the relevant crate/module and suggested areas
for deeper inspection.

| Perf symbol | Area | Notes |
| --- | --- | --- |
| `mi_malloc_aligned`, `clear_page_erms`, `_mi_free_delayed_block` | Allocation pressure | Focus on buffer reuse and arena allocation in module graph + codegen. |
| `rspack_core::module_graph::rollback::overlay_map::OverlayMap::get` | Module graph overlay | Review `crates/rspack_core/src/module_graph/rollback/overlay_map.rs` for hot lookup paths. |
| `rspack_core::utils::file_counter::FileCounter::add_files` | File dependency tracking | `crates/rspack_core/src/utils/file_counter/mod.rs` batching opportunities. |
| `ustr_fxhash::Ustr::from` | Identifier interning | Reduce repeated interning in module/dep creation. |
| `ExportsInfoGetter::prefetch_exports` | Export analysis | `crates/rspack_core/src/exports/exports_info_getter.rs` prefetch cache. |
| `GetSideEffectsConnectionStateCache::get` | Side-effects cache | `crates/rspack_core/src/artifacts/module_graph_cache_artifact.rs`. |
| `CodeSplitter::prepare` | Chunk graph build | `crates/rspack_core/src/compilation/build_chunk_graph/code_splitter.rs`. |
| `swc_ecma_minifier::compress::pure::Pure::visit_mut_expr` | Minifier | Check `rspack_plugin_swc_js_minimizer` and SWC config reuse. |
| `rspack_plugin_javascript::parser_plugin::inner_graph::*` | JS inner graph | Inspect inner-graph traversal and export usage analysis. |
| `ReplaceSource` in `URLPlugin` | Source replacement | `crates/rspack_plugin_javascript/src/plugin/url_plugin.rs` uses lossy conversions. |
