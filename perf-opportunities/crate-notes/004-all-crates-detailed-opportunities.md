# All Crates Detailed Opportunities Ledger

This ledger is a detailed, crate-by-crate backlog of remaining opportunities.
It complements the per-crate files and provides one auditable place to verify
coverage across all crates.

Legend:

- **Hot path likelihood**: high / medium / low
- **Primary opportunity**: highest-value next optimization direction
- **Validation**: minimal benchmark/profiling check to confirm impact

## Core and Infrastructure

- `node_binding`
  - Hot path likelihood: medium
  - Primary opportunity: reduce JS<->Rust marshaling/copying for large result payloads.
  - Validation: compare build wall time and Node heap allocations with/without batched API responses.

- `rspack`
  - Hot path likelihood: medium
  - Primary opportunity: avoid config/bootstrap conversion overhead in command startup and multi-compiler setup.
  - Validation: profile startup-only runs (`rspack --help`, minimal config build) and measure pre-make latency.

- `rspack_allocator`
  - Hot path likelihood: medium
  - Primary opportunity: tighten allocator telemetry and evaluate short-lived arena use for parser/module-graph temps.
  - Validation: peak RSS and allocation count deltas on react-10k builds.

- `rspack_binding_api`
  - Hot path likelihood: high
  - Primary opportunity: short-circuit untapped hook bridges and reduce repeated conversion in hot callbacks.
  - Validation: trace hook timings with and without no-tap fast path.

- `rspack_binding_build`
  - Hot path likelihood: low
  - Primary opportunity: keep build-time-only logic isolated from runtime paths.
  - Validation: ensure no runtime symbol inclusion regressions in release artifacts.

- `rspack_binding_builder`
  - Hot path likelihood: low
  - Primary opportunity: minimize generated conversion indirection in commonly used wrappers.
  - Validation: microbenchmark bridge call overhead.

- `rspack_binding_builder_macros`
  - Hot path likelihood: low
  - Primary opportunity: reduce generated clone/to_string calls in macro expansion outputs.
  - Validation: diff generated code metrics and bridge benchmark.

- `rspack_binding_builder_testing`
  - Hot path likelihood: low
  - Primary opportunity: none for runtime; keep test harness overhead isolated.
  - Validation: N/A (test-only).

- `rspack_browser`
  - Hot path likelihood: low
  - Primary opportunity: keep browser support shims branch-light and avoid repeated fallback checks.
  - Validation: browser-target benchmark for startup overhead.

- `rspack_browserslist`
  - Hot path likelihood: low-medium
  - Primary opportunity: cache normalized targets to avoid repeated parse/merge.
  - Validation: repeated build timings with identical target matrix.

- `rspack_cacheable`
  - Hot path likelihood: high (incremental/cache scenarios)
  - Primary opportunity: reduce intermediate buffers in serialize/deserialize and increase zero-copy archive usage.
  - Validation: cache load/store time and memory usage with persistent cache enabled.

- `rspack_cacheable_macros`
  - Hot path likelihood: medium
  - Primary opportunity: generate leaner serializers for common enums/structs.
  - Validation: compare serialized throughput before/after macro output refinement.

- `rspack_cacheable_test`
  - Hot path likelihood: low
  - Primary opportunity: none runtime-critical; keep fixtures representative.
  - Validation: N/A.

- `rspack_collections`
  - Hot path likelihood: high
  - Primary opportunity: tune map/set choices and reduce temporary materialization in iterate-only sites.
  - Validation: allocation and CPU deltas in module/chunk graph heavy phases.

- `rspack_core`
  - Hot path likelihood: very high
  - Primary opportunity: reduce make-phase main-thread bottlenecks, map materialization churn, and chunk graph bitset overhead.
  - Validation: phase timing comparison on react-10k and async-chunk synthetic workloads.

- `rspack_error`
  - Hot path likelihood: low-medium
  - Primary opportunity: defer expensive formatting on success paths and warning-heavy loops.
  - Validation: diagnostics-heavy build benchmark.

- `rspack_fs`
  - Hot path likelihood: medium
  - Primary opportunity: batch fs metadata calls and cache repeated path probes.
  - Validation: strace/fs activity style profiling and wall time in file-heavy builds.

- `rspack_futures`
  - Hot path likelihood: medium-high
  - Primary opportunity: tune task granularity to reduce spawn/fan-in overhead.
  - Validation: CPU utilization + latency in codegen/hash phases.

- `rspack_hash`
  - Hot path likelihood: medium
  - Primary opportunity: reduce hasher setup and digest formatting overhead in tight loops.
  - Validation: hash microbenchmarks and create_hash phase timing.

- `rspack_hook`
  - Hot path likelihood: medium-high
  - Primary opportunity: add stronger empty-tap fast paths and avoid needless dispatch.
  - Validation: hook timing traces with plugin sets of varying density.

- `rspack_ids`
  - Hot path likelihood: medium
  - Primary opportunity: reduce repeated sorting/hash work in ID assignment.
  - Validation: module/chunk ID pass timings at 10k modules.

- `rspack_javascript_compiler`
  - Hot path likelihood: high
  - Primary opportunity: reuse parser/compiler setup and reduce per-module AST setup churn.
  - Validation: parser phase timing and allocation profile on react-10k.

- `rspack_loader_lightningcss`
  - Hot path likelihood: medium
  - Primary opportunity: cache transform outputs for unchanged inputs; defer source map heavy work.
  - Validation: css-heavy case timings with cache on/off.

- `rspack_loader_preact_refresh`
  - Hot path likelihood: low
  - Primary opportunity: maintain strict dev-only gating.
  - Validation: confirm zero inclusion/overhead in production builds.

- `rspack_loader_react_refresh`
  - Hot path likelihood: low
  - Primary opportunity: maintain strict dev-only gating and avoid fallback checks in prod.
  - Validation: production bundle diff + startup timing.

- `rspack_loader_runner`
  - Hot path likelihood: high
  - Primary opportunity: minimize content conversion churn (bytes<->string) and per-loader setup overhead.
  - Validation: loader pipeline span timings on large module sets.

- `rspack_loader_swc`
  - Hot path likelihood: high
  - Primary opportunity: reduce transform setup duplication and source-map overhead in bulk transforms.
  - Validation: SWC loader phase timing + allocation comparison.

- `rspack_loader_testing`
  - Hot path likelihood: low
  - Primary opportunity: N/A runtime; keep test helper overhead local.
  - Validation: N/A.

- `rspack_location`
  - Hot path likelihood: low-medium
  - Primary opportunity: avoid location object creation in hot loops unless diagnostics requested.
  - Validation: diagnostics off/on overhead measurements.

- `rspack_macros`
  - Hot path likelihood: low-medium
  - Primary opportunity: reduce generated clones/formats in expanded code paths.
  - Validation: generated code diff + affected microbenchmarks.

- `rspack_macros_test`
  - Hot path likelihood: low
  - Primary opportunity: N/A runtime.
  - Validation: N/A.

- `rspack_napi`
  - Hot path likelihood: medium
  - Primary opportunity: optimize conversion pathways and reduce copying for large buffers.
  - Validation: bridge microbenchmark and end-to-end CLI timings with heavy stats output.

- `rspack_napi_macros`
  - Hot path likelihood: low-medium
  - Primary opportunity: generate zero-copy friendly wrappers by default where safe.
  - Validation: benchmark macro-generated bridge functions.

- `rspack_paths`
  - Hot path likelihood: medium
  - Primary opportunity: reduce repeated normalization and UTF-8/path conversion churn.
  - Validation: resolver-heavy build profiling.

## Plugins

- `rspack_plugin_asset`
  - Hot path likelihood: medium
  - Primary opportunity: dedupe hash/metadata recomputation for unchanged assets.
  - Validation: asset-heavy build diff in process_assets/create_hash phases.

- `rspack_plugin_banner`
  - Hot path likelihood: low
  - Primary opportunity: preserve no-op fast path when banner disabled.
  - Validation: plugin-enabled vs disabled overhead check.

- `rspack_plugin_case_sensitive`
  - Hot path likelihood: low-medium
  - Primary opportunity: cache case checks per path segment.
  - Validation: large path graph with repeated imports.

- `rspack_plugin_circular_dependencies`
  - Hot path likelihood: low-medium
  - Primary opportunity: bound traversal and prune early on acyclic subgraphs.
  - Validation: cycle-dense graph synthetic tests.

- `rspack_plugin_copy`
  - Hot path likelihood: medium
  - Primary opportunity: batch copy/stat operations and reduce per-file sync points.
  - Validation: copy-heavy project timings.

- `rspack_plugin_css`
  - Hot path likelihood: medium-high
  - Primary opportunity: reduce clone-heavy module ordering and repeated sorting.
  - Validation: css ordering span timings on chunk-rich builds.

- `rspack_plugin_css_chunking`
  - Hot path likelihood: medium
  - Primary opportunity: memoize chunk-group ordering decisions across unchanged graphs.
  - Validation: repeated build timings with stable CSS graph.

- `rspack_plugin_devtool`
  - Hot path likelihood: medium
  - Primary opportunity: aggressively skip source map work when not required.
  - Validation: devtool matrix benchmark.

- `rspack_plugin_dll`
  - Hot path likelihood: medium
  - Primary opportunity: cache manifest parsing and reduce repeated symbol map construction.
  - Validation: dll build series.

- `rspack_plugin_dynamic_entry`
  - Hot path likelihood: low-medium
  - Primary opportunity: cache dynamic entry resolution results.
  - Validation: dynamic entry stress case.

- `rspack_plugin_ensure_chunk_conditions`
  - Hot path likelihood: low-medium
  - Primary opportunity: cache condition checks and avoid full rescans.
  - Validation: chunk condition plugin microbenchmark.

- `rspack_plugin_entry`
  - Hot path likelihood: low-medium
  - Primary opportunity: reduce entry descriptor rebuilding in multi-entry scenarios.
  - Validation: many-entry benchmark.

- `rspack_plugin_esm_library`
  - Hot path likelihood: medium-high
  - Primary opportunity: avoid repeated export chain analysis and re-parse in library mode.
  - Validation: esm library output benchmark at scale.

- `rspack_plugin_externals`
  - Hot path likelihood: medium
  - Primary opportunity: cache externals matching by (request, context).
  - Validation: externals-heavy build timings.

- `rspack_plugin_extract_css`
  - Hot path likelihood: medium
  - Primary opportunity: reduce concatenation/string copy pressure during CSS extraction.
  - Validation: css extract pipeline timings.

- `rspack_plugin_hmr`
  - Hot path likelihood: low-medium
  - Primary opportunity: isolate HMR-only paths from cold production build.
  - Validation: prod build overhead with plugin present vs absent.

- `rspack_plugin_html`
  - Hot path likelihood: medium
  - Primary opportunity: cache template parse/render intermediates.
  - Validation: multi-page html generation benchmark.

- `rspack_plugin_ignore`
  - Hot path likelihood: low
  - Primary opportunity: ensure constant-time fast-path matching.
  - Validation: ignore-heavy import graph test.

- `rspack_plugin_javascript`
  - Hot path likelihood: very high
  - Primary opportunity: consolidate AST passes, optimize parser plugin dispatch, reduce tree-shaking clone churn.
  - Validation: parser + optimize-dependencies phase comparisons on react-10k.

- `rspack_plugin_json`
  - Hot path likelihood: medium
  - Primary opportunity: minimize unnecessary string conversions in parse/codegen.
  - Validation: large JSON module suite benchmark.

- `rspack_plugin_lazy_compilation`
  - Hot path likelihood: low-medium
  - Primary opportunity: keep disabled-state overhead near zero.
  - Validation: baseline cold build with plugin loaded but inactive.

- `rspack_plugin_library`
  - Hot path likelihood: medium
  - Primary opportunity: reduce repeated library export/render formatting work.
  - Validation: library-target build benchmark.

- `rspack_plugin_lightning_css_minimizer`
  - Hot path likelihood: medium
  - Primary opportunity: reduce minifier allocation churn and tune parallel chunking.
  - Validation: css minification-heavy benchmark.

- `rspack_plugin_limit_chunk_count`
  - Hot path likelihood: low-medium
  - Primary opportunity: prune candidate reevaluation loops.
  - Validation: many-chunk synthetic workload.

- `rspack_plugin_merge_duplicate_chunks`
  - Hot path likelihood: medium
  - Primary opportunity: improve duplicate detection indexing and reuse fingerprints.
  - Validation: duplicate-chunk-heavy case timings.

- `rspack_plugin_mf`
  - Hot path likelihood: high
  - Primary opportunity: reduce manifest/runtime template generation overhead and lock contention.
  - Validation: federation-heavy build and runtime module timing.

- `rspack_plugin_module_info_header`
  - Hot path likelihood: low-medium
  - Primary opportunity: gate formatting and avoid per-module string churn when disabled.
  - Validation: header enabled/disabled diff.

- `rspack_plugin_module_replacement`
  - Hot path likelihood: low-medium
  - Primary opportunity: cache replacement decisions.
  - Validation: replacement-rule-heavy test.

- `rspack_plugin_no_emit_on_errors`
  - Hot path likelihood: low
  - Primary opportunity: keep checks branch-light and allocation-free.
  - Validation: negligible-overhead assertion.

- `rspack_plugin_progress`
  - Hot path likelihood: low-medium
  - Primary opportunity: throttle progress updates and formatting.
  - Validation: high-module-count build with progress on/off.

- `rspack_plugin_real_content_hash`
  - Hot path likelihood: medium
  - Primary opportunity: reuse computed hashes and reduce remap overhead.
  - Validation: content-hash-heavy asset test.

- `rspack_plugin_remove_duplicate_modules`
  - Hot path likelihood: medium
  - Primary opportunity: improve dedupe search/index structures.
  - Validation: duplicate-module synthetic benchmark.

- `rspack_plugin_remove_empty_chunks`
  - Hot path likelihood: low-medium
  - Primary opportunity: stronger early exits before full graph pass.
  - Validation: empty-chunk-heavy graph test.

- `rspack_plugin_rsc`
  - Hot path likelihood: medium-high
  - Primary opportunity: cache transform outputs and reduce repeated analysis.
  - Validation: RSC project rebuild timings.

- `rspack_plugin_rsdoctor`
  - Hot path likelihood: medium (when enabled)
  - Primary opportunity: reduce instrumentation overhead and payload size.
  - Validation: rsdoctor enabled build timing + output size.

- `rspack_plugin_rslib`
  - Hot path likelihood: low-medium
  - Primary opportunity: reduce integration bridge overhead.
  - Validation: rslib mode benchmark.

- `rspack_plugin_rstest`
  - Hot path likelihood: low
  - Primary opportunity: keep instrumentation off production hot paths.
  - Validation: N/A.

- `rspack_plugin_runtime`
  - Hot path likelihood: high
  - Primary opportunity: cache runtime module rendering by requirement signature and reduce format-heavy paths.
  - Validation: runtime generation phase timing in multi-runtime builds.

- `rspack_plugin_runtime_chunk`
  - Hot path likelihood: low-medium
  - Primary opportunity: avoid extra traversal when runtime chunk strategy unchanged.
  - Validation: runtime-chunk on/off comparison.

- `rspack_plugin_schemes`
  - Hot path likelihood: medium
  - Primary opportunity: cache scheme dispatch and parse results.
  - Validation: URL/scheme-heavy import graph benchmark.

- `rspack_plugin_size_limits`
  - Hot path likelihood: low
  - Primary opportunity: short-circuit checks when thresholds disabled/default.
  - Validation: overhead check at large chunk counts.

- `rspack_plugin_split_chunks`
  - Hot path likelihood: high
  - Primary opportunity: improve candidate selection algorithm and reduce recomputation.
  - Validation: chunk-heavy async graph timings.

- `rspack_plugin_sri`
  - Hot path likelihood: medium
  - Primary opportunity: parallelize and cache integrity hash computations.
  - Validation: asset-rich build with SRI enabled.

- `rspack_plugin_swc_js_minimizer`
  - Hot path likelihood: medium-high
  - Primary opportunity: improve minifier task partitioning and reduce temporary allocation.
  - Validation: minification phase timing.

- `rspack_plugin_wasm`
  - Hot path likelihood: medium
  - Primary opportunity: cache module metadata analysis.
  - Validation: wasm module benchmark.

- `rspack_plugin_web_worker_template`
  - Hot path likelihood: low
  - Primary opportunity: cache template fragments.
  - Validation: worker-heavy build.

- `rspack_plugin_worker`
  - Hot path likelihood: low-medium
  - Primary opportunity: cache worker entry resolution path.
  - Validation: many worker entry scenario timings.

## Remaining Utility/Support Crates

- `rspack_regex`
  - Hot path likelihood: medium
  - Primary opportunity: precompile and cache regex objects in repeated matching paths.
  - Validation: regex-heavy resolver/plugin tests.

- `rspack_storage`
  - Hot path likelihood: high (incremental)
  - Primary opportunity: reduce serialization overhead and improve write batching/mmap read paths.
  - Validation: persistent cache rebuild benchmark.

- `rspack_tasks`
  - Hot path likelihood: medium
  - Primary opportunity: reduce boxing/spawn overhead in task wrappers.
  - Validation: spawn-heavy microbenchmarks.

- `rspack_tools`
  - Hot path likelihood: low
  - Primary opportunity: keep debug helpers out of production execution paths.
  - Validation: binary size/path audit.

- `rspack_tracing`
  - Hot path likelihood: medium
  - Primary opportunity: reduce trace-disabled overhead and formatting costs.
  - Validation: tracing on/off overhead check.

- `rspack_tracing_perfetto`
  - Hot path likelihood: medium (when enabled)
  - Primary opportunity: batch writer flushes and reduce synchronization overhead.
  - Validation: trace write throughput benchmark.

- `rspack_util`
  - Hot path likelihood: medium
  - Primary opportunity: identify helper-induced hidden allocations and offer allocation-light alternatives.
  - Validation: helper callsite profiling in hot phases.

- `rspack_watcher`
  - Hot path likelihood: medium
  - Primary opportunity: reduce event burst amplification and path-stat overhead.
  - Validation: large repo watch/rebuild latency benchmark.

- `rspack_workspace`
  - Hot path likelihood: low
  - Primary opportunity: keep workspace helper calls out of per-module loops.
  - Validation: startup/build orchestration profiling.

- `swc_plugin_import`
  - Hot path likelihood: medium
  - Primary opportunity: cache option parsing and avoid repeated transform setup.
  - Validation: import-transform-heavy benchmark.

- `swc_plugin_ts_collector`
  - Hot path likelihood: low-medium
  - Primary opportunity: skip deep traversal when collector output unused.
  - Validation: TS collector enabled/disabled benchmark.

## Coverage Confirmation

- Crates in workspace (`crates/*/Cargo.toml`): 92
- Crates explicitly listed in this ledger: 92
- Detailed per-crate note files under `perf-opportunities/crate-notes`: present
