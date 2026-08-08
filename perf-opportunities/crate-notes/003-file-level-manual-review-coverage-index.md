# File-Level Manual Review Coverage Index

This index is the traceability ledger for the manual per-crate Rust review pass.
It records crate coverage, Rust file counts, and the corresponding note file
where opportunities are documented.

## Coverage Summary

- Workspace crates: 92
- Crates with note file: 92
- Total Rust files under `crates/*`: 1129
- Coverage status: all crates covered in `crate-notes`, with file-level counts tracked below.

## Crate Coverage Table

| Crate | Rust files | Note file | Manual pass status |
|---|---:|---|---|
| `node_binding` | 2 | `perf-opportunities/crate-notes/node_binding.md` | reviewed |
| `rspack` | 11 | `perf-opportunities/crate-notes/rspack.md` | reviewed |
| `rspack_allocator` | 1 | `perf-opportunities/crate-notes/rspack_allocator.md` | reviewed |
| `rspack_binding_api` | 125 | `perf-opportunities/crate-notes/rspack_binding_api.md` | reviewed |
| `rspack_binding_build` | 1 | `perf-opportunities/crate-notes/rspack_binding_build.md` | reviewed |
| `rspack_binding_builder` | 1 | `perf-opportunities/crate-notes/rspack_binding_builder.md` | reviewed |
| `rspack_binding_builder_macros` | 2 | `perf-opportunities/crate-notes/rspack_binding_builder_macros.md` | reviewed |
| `rspack_binding_builder_testing` | 2 | `perf-opportunities/crate-notes/rspack_binding_builder_testing.md` | reviewed |
| `rspack_browser` | 3 | `perf-opportunities/crate-notes/rspack_browser.md` | reviewed |
| `rspack_browserslist` | 3 | `perf-opportunities/crate-notes/rspack_browserslist.md` | reviewed |
| `rspack_cacheable` | 36 | `perf-opportunities/crate-notes/rspack_cacheable.md` | reviewed |
| `rspack_cacheable_macros` | 7 | `perf-opportunities/crate-notes/rspack_cacheable_macros.md` | reviewed |
| `rspack_cacheable_test` | 50 | `perf-opportunities/crate-notes/rspack_cacheable_test.md` | reviewed |
| `rspack_collections` | 3 | `perf-opportunities/crate-notes/rspack_collections.md` | reviewed |
| `rspack_core` | 236 | `perf-opportunities/crate-notes/rspack_core.md` | reviewed |
| `rspack_error` | 14 | `perf-opportunities/crate-notes/rspack_error.md` | reviewed |
| `rspack_fs` | 9 | `perf-opportunities/crate-notes/rspack_fs.md` | reviewed |
| `rspack_futures` | 2 | `perf-opportunities/crate-notes/rspack_futures.md` | reviewed |
| `rspack_hash` | 1 | `perf-opportunities/crate-notes/rspack_hash.md` | reviewed |
| `rspack_hook` | 1 | `perf-opportunities/crate-notes/rspack_hook.md` | reviewed |
| `rspack_ids` | 9 | `perf-opportunities/crate-notes/rspack_ids.md` | reviewed |
| `rspack_javascript_compiler` | 10 | `perf-opportunities/crate-notes/rspack_javascript_compiler.md` | reviewed |
| `rspack_loader_lightningcss` | 3 | `perf-opportunities/crate-notes/rspack_loader_lightningcss.md` | reviewed |
| `rspack_loader_preact_refresh` | 2 | `perf-opportunities/crate-notes/rspack_loader_preact_refresh.md` | reviewed |
| `rspack_loader_react_refresh` | 2 | `perf-opportunities/crate-notes/rspack_loader_react_refresh.md` | reviewed |
| `rspack_loader_runner` | 7 | `perf-opportunities/crate-notes/rspack_loader_runner.md` | reviewed |
| `rspack_loader_swc` | 11 | `perf-opportunities/crate-notes/rspack_loader_swc.md` | reviewed |
| `rspack_loader_testing` | 1 | `perf-opportunities/crate-notes/rspack_loader_testing.md` | reviewed |
| `rspack_location` | 1 | `perf-opportunities/crate-notes/rspack_location.md` | reviewed |
| `rspack_macros` | 6 | `perf-opportunities/crate-notes/rspack_macros.md` | reviewed |
| `rspack_macros_test` | 6 | `perf-opportunities/crate-notes/rspack_macros_test.md` | reviewed |
| `rspack_napi` | 14 | `perf-opportunities/crate-notes/rspack_napi.md` | reviewed |
| `rspack_napi_macros` | 3 | `perf-opportunities/crate-notes/rspack_napi_macros.md` | reviewed |
| `rspack_paths` | 1 | `perf-opportunities/crate-notes/rspack_paths.md` | reviewed |
| `rspack_plugin_asset` | 2 | `perf-opportunities/crate-notes/rspack_plugin_asset.md` | reviewed |
| `rspack_plugin_banner` | 1 | `perf-opportunities/crate-notes/rspack_plugin_banner.md` | reviewed |
| `rspack_plugin_case_sensitive` | 1 | `perf-opportunities/crate-notes/rspack_plugin_case_sensitive.md` | reviewed |
| `rspack_plugin_circular_dependencies` | 1 | `perf-opportunities/crate-notes/rspack_plugin_circular_dependencies.md` | reviewed |
| `rspack_plugin_copy` | 1 | `perf-opportunities/crate-notes/rspack_plugin_copy.md` | reviewed |
| `rspack_plugin_css` | 14 | `perf-opportunities/crate-notes/rspack_plugin_css.md` | reviewed |
| `rspack_plugin_css_chunking` | 1 | `perf-opportunities/crate-notes/rspack_plugin_css_chunking.md` | reviewed |
| `rspack_plugin_devtool` | 8 | `perf-opportunities/crate-notes/rspack_plugin_devtool.md` | reviewed |
| `rspack_plugin_dll` | 13 | `perf-opportunities/crate-notes/rspack_plugin_dll.md` | reviewed |
| `rspack_plugin_dynamic_entry` | 1 | `perf-opportunities/crate-notes/rspack_plugin_dynamic_entry.md` | reviewed |
| `rspack_plugin_ensure_chunk_conditions` | 1 | `perf-opportunities/crate-notes/rspack_plugin_ensure_chunk_conditions.md` | reviewed |
| `rspack_plugin_entry` | 1 | `perf-opportunities/crate-notes/rspack_plugin_entry.md` | reviewed |
| `rspack_plugin_esm_library` | 11 | `perf-opportunities/crate-notes/rspack_plugin_esm_library.md` | reviewed |
| `rspack_plugin_externals` | 5 | `perf-opportunities/crate-notes/rspack_plugin_externals.md` | reviewed |
| `rspack_plugin_extract_css` | 6 | `perf-opportunities/crate-notes/rspack_plugin_extract_css.md` | reviewed |
| `rspack_plugin_hmr` | 2 | `perf-opportunities/crate-notes/rspack_plugin_hmr.md` | reviewed |
| `rspack_plugin_html` | 9 | `perf-opportunities/crate-notes/rspack_plugin_html.md` | reviewed |
| `rspack_plugin_ignore` | 1 | `perf-opportunities/crate-notes/rspack_plugin_ignore.md` | reviewed |
| `rspack_plugin_javascript` | 151 | `perf-opportunities/crate-notes/rspack_plugin_javascript.md` | reviewed |
| `rspack_plugin_json` | 3 | `perf-opportunities/crate-notes/rspack_plugin_json.md` | reviewed |
| `rspack_plugin_lazy_compilation` | 7 | `perf-opportunities/crate-notes/rspack_plugin_lazy_compilation.md` | reviewed |
| `rspack_plugin_library` | 11 | `perf-opportunities/crate-notes/rspack_plugin_library.md` | reviewed |
| `rspack_plugin_lightning_css_minimizer` | 1 | `perf-opportunities/crate-notes/rspack_plugin_lightning_css_minimizer.md` | reviewed |
| `rspack_plugin_limit_chunk_count` | 2 | `perf-opportunities/crate-notes/rspack_plugin_limit_chunk_count.md` | reviewed |
| `rspack_plugin_merge_duplicate_chunks` | 1 | `perf-opportunities/crate-notes/rspack_plugin_merge_duplicate_chunks.md` | reviewed |
| `rspack_plugin_mf` | 45 | `perf-opportunities/crate-notes/rspack_plugin_mf.md` | reviewed |
| `rspack_plugin_module_info_header` | 1 | `perf-opportunities/crate-notes/rspack_plugin_module_info_header.md` | reviewed |
| `rspack_plugin_module_replacement` | 3 | `perf-opportunities/crate-notes/rspack_plugin_module_replacement.md` | reviewed |
| `rspack_plugin_no_emit_on_errors` | 1 | `perf-opportunities/crate-notes/rspack_plugin_no_emit_on_errors.md` | reviewed |
| `rspack_plugin_progress` | 1 | `perf-opportunities/crate-notes/rspack_plugin_progress.md` | reviewed |
| `rspack_plugin_real_content_hash` | 2 | `perf-opportunities/crate-notes/rspack_plugin_real_content_hash.md` | reviewed |
| `rspack_plugin_remove_duplicate_modules` | 1 | `perf-opportunities/crate-notes/rspack_plugin_remove_duplicate_modules.md` | reviewed |
| `rspack_plugin_remove_empty_chunks` | 1 | `perf-opportunities/crate-notes/rspack_plugin_remove_empty_chunks.md` | reviewed |
| `rspack_plugin_rsc` | 16 | `perf-opportunities/crate-notes/rspack_plugin_rsc.md` | reviewed |
| `rspack_plugin_rsdoctor` | 6 | `perf-opportunities/crate-notes/rspack_plugin_rsdoctor.md` | reviewed |
| `rspack_plugin_rslib` | 7 | `perf-opportunities/crate-notes/rspack_plugin_rslib.md` | reviewed |
| `rspack_plugin_rstest` | 8 | `perf-opportunities/crate-notes/rspack_plugin_rstest.md` | reviewed |
| `rspack_plugin_runtime` | 63 | `perf-opportunities/crate-notes/rspack_plugin_runtime.md` | reviewed |
| `rspack_plugin_runtime_chunk` | 1 | `perf-opportunities/crate-notes/rspack_plugin_runtime_chunk.md` | reviewed |
| `rspack_plugin_schemes` | 6 | `perf-opportunities/crate-notes/rspack_plugin_schemes.md` | reviewed |
| `rspack_plugin_size_limits` | 1 | `perf-opportunities/crate-notes/rspack_plugin_size_limits.md` | reviewed |
| `rspack_plugin_split_chunks` | 13 | `perf-opportunities/crate-notes/rspack_plugin_split_chunks.md` | reviewed |
| `rspack_plugin_sri` | 7 | `perf-opportunities/crate-notes/rspack_plugin_sri.md` | reviewed |
| `rspack_plugin_swc_js_minimizer` | 1 | `perf-opportunities/crate-notes/rspack_plugin_swc_js_minimizer.md` | reviewed |
| `rspack_plugin_wasm` | 7 | `perf-opportunities/crate-notes/rspack_plugin_wasm.md` | reviewed |
| `rspack_plugin_web_worker_template` | 1 | `perf-opportunities/crate-notes/rspack_plugin_web_worker_template.md` | reviewed |
| `rspack_plugin_worker` | 1 | `perf-opportunities/crate-notes/rspack_plugin_worker.md` | reviewed |
| `rspack_regex` | 4 | `perf-opportunities/crate-notes/rspack_regex.md` | reviewed |
| `rspack_storage` | 28 | `perf-opportunities/crate-notes/rspack_storage.md` | reviewed |
| `rspack_tasks` | 1 | `perf-opportunities/crate-notes/rspack_tasks.md` | reviewed |
| `rspack_tools` | 9 | `perf-opportunities/crate-notes/rspack_tools.md` | reviewed |
| `rspack_tracing` | 4 | `perf-opportunities/crate-notes/rspack_tracing.md` | reviewed |
| `rspack_tracing_perfetto` | 2 | `perf-opportunities/crate-notes/rspack_tracing_perfetto.md` | reviewed |
| `rspack_util` | 24 | `perf-opportunities/crate-notes/rspack_util.md` | reviewed |
| `rspack_watcher` | 12 | `perf-opportunities/crate-notes/rspack_watcher.md` | reviewed |
| `rspack_workspace` | 2 | `perf-opportunities/crate-notes/rspack_workspace.md` | reviewed |
| `swc_plugin_import` | 5 | `perf-opportunities/crate-notes/swc_plugin_import.md` | reviewed |
| `swc_plugin_ts_collector` | 4 | `perf-opportunities/crate-notes/swc_plugin_ts_collector.md` | reviewed |

## Notes

- This coverage index tracks file-count completeness and review status by crate.
- Detailed opportunities are in:
  - `perf-opportunities/crate-notes/004-all-crates-detailed-opportunities.md`
  - `perf-opportunities/crate-notes/001-all-crates-remaining-opportunities.md`
