# All Crates: Remaining Performance Opportunities

This file tracks remaining opportunities crate-by-crate, one line per crate.
Priority is relative to large workloads (react-10k style and chunk-heavy apps).
The list is consolidated from:

- direct per-crate note review (`crate-notes/<crate>.md`)
- react-10k profiling evidence in `survey-0f36/*` and `43-react-10k-actual-profile.md`
- independent static signal scan across every Rust file in all crates

## Tier 1 (Highest Remaining Impact)

- `rspack_core`: reduce main-thread graph update bottlenecks, avoid map materialization churn, optimize chunk-graph bitset paths, and improve pass-level reuse.
- `rspack_plugin_javascript`: cut multi-pass AST walks, reduce inner-graph/tree-shaking clone work, and optimize parser plugin dispatch.
- `rspack_javascript_compiler`: reuse parser/config state and reduce per-module setup and allocation churn.
- `rspack_loader_swc`: reduce transform setup duplication and source-map overhead on large module sets.
- `rspack_plugin_split_chunks`: reduce candidate recomputation complexity and improve cache-group selection efficiency.
- `rspack_plugin_runtime`: cache runtime template fragments by requirements signature and reduce repeated rendering.
- `rspack_storage`: reduce serialization and write amplification, improve persistent cache IO and invalidation precision.
- `rspack_binding_api`: add empty-hook short-circuiting and reduce JS<->Rust marshaling overhead in hot hooks.
- `node_binding`: minimize NAPI boundary copies and conversion overhead in compile stats/events paths.
- `rspack_cacheable`: improve zero-copy/archive usage and reduce intermediate buffers during cache materialization.

## Tier 2 (Medium Impact / Medium Effort)

- `rspack_plugin_css`: optimize module ordering and reduce clone-heavy ordering logic.
- `rspack_plugin_css_chunking`: reduce repeated chunk-group traversals and memoize ordering decisions.
- `rspack_plugin_swc_js_minimizer`: reduce minify-time allocations and increase work partition efficiency.
- `rspack_plugin_lightning_css_minimizer`: reduce CSS minifier overhead and cache unchanged assets.
- `rspack_plugin_mf`: reduce manifest/runtime generation duplication and large string/template churn.
- `rspack_plugin_esm_library`: avoid repeated export chain analysis and re-parse overhead.
- `rspack_plugin_rsdoctor`: keep diagnostics instrumentation behind low-cost gates in non-analysis builds.
- `rspack_plugin_rsc`: cache expensive transform outputs for unchanged modules.
- `rspack_plugin_extract_css`: reduce asset render/concatenation copy pressure.
- `rspack_plugin_library`: reduce repeated render formatting and export bookkeeping scans.
- `rspack_plugin_json`: avoid unnecessary UTF-8/string conversion in parse/generate cycle.
- `rspack_loader_runner`: keep content as bytes where possible and reduce lossy conversion paths.
- `rspack_loader_lightningcss`: cache parse/transform artifacts for unchanged CSS inputs.
- `rspack_futures`: tune task granularity and reduce spawn/fan-in overhead in short jobs.
- `rspack_hash`: remove avoidable hasher setup/digest allocation overhead in hot loops.
- `rspack_collections`: replace concurrent maps where unnecessary and reduce temporary map allocation.
- `rspack_ids`: reduce repeated ID/hash recomputation and improve memoization in ID assignment stages.
- `rspack_watcher`: reduce stat storm/re-scan overhead and improve debounce behavior on large trees.
- `rspack_plugin_real_content_hash`: reduce repeated hashing and improve digest reuse across assets.
- `rspack_plugin_merge_duplicate_chunks`: reduce chunk fingerprint recomputation and graph scan cost.
- `rspack_plugin_remove_duplicate_modules`: reduce duplicate-detection scan complexity with better indexing.
- `rspack_plugin_remove_empty_chunks`: add stronger early-exit conditions to avoid whole-graph passes.
- `rspack_plugin_limit_chunk_count`: reduce repeated candidate evaluation passes in limit enforcement.
- `rspack_plugin_ensure_chunk_conditions`: cache condition checks to avoid repetitive scans.
- `rspack_plugin_worker`: cache worker entry resolution and reduce repeated dependency wiring.
- `rspack_plugin_web_worker_template`: reduce template generation duplication across similar workers.

## Tier 3 (Lower Impact / Mostly Fast-Path Work)

- `rspack`: keep top-level coordination thin and avoid extra conversion/allocation glue.
- `rspack_allocator`: continue allocator telemetry and pool/arena experiments for short-lived objects.
- `rspack_binding_build`: build-time crate; keep runtime leakage zero.
- `rspack_binding_builder`: ensure generated conversion paths avoid unnecessary clones.
- `rspack_binding_builder_macros`: minimize generated abstraction overhead in hot call sites.
- `rspack_binding_builder_testing`: test-only crate; no runtime hotspot expected.
- `rspack_browser`: keep browser compatibility layers thin and avoid redundant wrappers.
- `rspack_browserslist`: cache parsed targets and avoid repeated parse/normalize work.
- `rspack_cacheable_macros`: keep generated serializers compact and allocation-aware.
- `rspack_cacheable_test`: test crate; no production perf impact.
- `rspack_error`: avoid expensive formatting on success paths; keep error construction lazy.
- `rspack_fs`: batch filesystem metadata reads and cache hot path lookups where safe.
- `rspack_hook`: add stronger empty-tap fast paths and reduce dispatch overhead.
- `rspack_loader_preact_refresh`: ensure strict dev-only gating.
- `rspack_loader_react_refresh`: ensure strict dev-only gating.
- `rspack_loader_testing`: test crate; no runtime hotspot expected.
- `rspack_location`: keep source-location metadata disabled/minimal outside diagnostics.
- `rspack_macros`: avoid generated code that allocates in hot loops.
- `rspack_macros_test`: test crate; no production perf impact.
- `rspack_napi`: optimize conversion paths and avoid copying large buffers/strings.
- `rspack_napi_macros`: improve generated binding glue for zero-copy typed buffers.
- `rspack_paths`: reduce path normalization churn and conversion frequency.
- `rspack_plugin_asset`: reduce repeated hash/metadata work for unchanged assets.
- `rspack_plugin_banner`: keep banner injection path skipped unless enabled.
- `rspack_plugin_case_sensitive`: cache path checks and avoid repeated filesystem probes.
- `rspack_plugin_circular_dependencies`: bound traversal cost and gate aggressively in prod builds.
- `rspack_plugin_copy`: batch copy/stat operations and reduce per-file overhead.
- `rspack_plugin_devtool`: minimize source-map work when disabled and improve map emission batching.
- `rspack_plugin_dll`: cache manifest parsing and avoid repeated full-scan generation.
- `rspack_plugin_dynamic_entry`: cache dynamic entry resolution and avoid repeated normalization.
- `rspack_plugin_entry`: avoid repeated entry descriptor rebuilding.
- `rspack_plugin_externals`: cache externals match results by request/context key.
- `rspack_plugin_hmr`: keep HMR-only logic isolated from cold build paths.
- `rspack_plugin_html`: cache template render intermediates and minify repeated operations.
- `rspack_plugin_ignore`: ensure O(1)-style fast path for common ignore checks.
- `rspack_plugin_lazy_compilation`: keep off-path overhead negligible when feature disabled.
- `rspack_plugin_module_info_header`: reduce string formatting overhead and gate by mode.
- `rspack_plugin_module_replacement`: cache replacement rules and short-circuit no-op requests.
- `rspack_plugin_no_emit_on_errors`: keep checks lightweight and non-allocating.
- `rspack_plugin_progress`: reduce high-frequency progress update overhead.
- `rspack_plugin_rslib`: keep integration hooks lean and avoid repeated bridge calls.
- `rspack_plugin_rstest`: test integration crate; keep instrumentation gated.
- `rspack_plugin_runtime_chunk`: minimize extra traversal when runtime chunk strategy unchanged.
- `rspack_plugin_schemes`: cache scheme handler resolution and URL parse outputs.
- `rspack_plugin_size_limits`: avoid expensive size checks unless limits configured.
- `rspack_plugin_sri`: optimize integrity hash computation and parallelize when many assets exist.
- `rspack_plugin_wasm`: cache wasm metadata parsing and reduce re-analysis on unchanged modules.
- `rspack_regex`: precompile/cache regex objects where repeatedly used.
- `rspack_tasks`: keep spawn wrappers minimal and avoid extra boxing in hot queues.
- `rspack_tools`: non-hot helper crate; avoid runtime inclusion of heavy debug paths.
- `rspack_tracing`: ensure low overhead when tracing disabled; avoid costly formatting.
- `rspack_tracing_perfetto`: keep perfetto emission behind feature/flag and batch writes.
- `rspack_util`: audit utility helpers for hidden allocations in hot loops.
- `rspack_workspace`: workspace helper crate; avoid per-module conversion work.
- `swc_plugin_import`: cache option parsing and avoid repeated transform setup work.
- `swc_plugin_ts_collector`: skip deep traversal when TS metadata collection is not needed.

## Coverage Check

- Source crates discovered from `crates/*/Cargo.toml`: 92
- Crates listed in this file: 92
- Per-crate note files present in `perf-opportunities/crate-notes`: 92
