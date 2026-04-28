# Independent Crate Static-Signal Metrics

This is a whole-repo static scan across all Rust files in `crates/*` to surface
potential performance-pressure indicators by crate.

## Method

For each crate, count occurrences of:

- `.clone(` (clone pressure proxy)
- `.collect::<` (materialization pressure proxy)
- `DashMap`/`DashSet` (concurrency map usage)
- `Mutex`/`RwLock` (lock contention risk)
- `Box<dyn ...>` (dynamic dispatch/indirection density)
- `async_trait` (async trait object overhead risk)
- `.to_string(` and `format!(` (string allocation pressure)
- `Arc<` (shared ownership overhead)

## Top-signal crates (selected)

- `rspack_core`: very high clone/collect + dynamic dispatch + async trait usage.
- `rspack_plugin_javascript`: high clone + formatting + string-heavy parser/codegen paths.
- `rspack_binding_api`: high interop/async/dynamic-dispatch signal.
- `rspack_storage`: high string/format/arc density in storage/cache surfaces.
- `rspack_plugin_runtime`: high template/format density.
- `rspack_plugin_mf`: high clone + lock + async usage.

## Raw Metrics (TSV)

```text
crate	rs_files	loc	clone	collect	dashmap	mutex	boxed_dyn	async_trait	to_string	format	arc
rspack_core	236	55349	713	180	16	21	72	94	273	339	214
rspack_plugin_javascript	151	39586	471	70	0	5	17	1	265	290	33
rspack_binding_api	125	23376	276	146	0	19	11	69	197	34	47
rspack_storage	28	6442	138	53	0	7	5	24	61	107	78
rspack_plugin_runtime	63	7543	27	10	0	0	19	47	141	146	2
rspack_plugin_mf	45	7675	196	21	0	21	11	31	88	71	30
rspack_plugin_esm_library	11	5006	162	26	0	2	5	1	24	89	4
rspack_loader_swc	11	4410	126	5	0	3	0	1	16	12	3
rspack_plugin_css	14	3330	38	11	0	0	5	2	48	36	3
rspack_plugin_split_chunks	13	2718	30	27	4	0	0	0	4	6	5
rspack_javascript_compiler	10	1615	28	9	0	0	1	0	11	7	15
rspack_loader_runner	7	1860	8	2	0	0	1	20	24	0	29
rspack_cacheable	36	2772	3	9	6	0	0	0	3	4	5
rspack_collections	3	360	1	0	8	0	0	0	1	0	0
rspack_futures	2	270	0	0	0	0	2	0	0	0	0
rspack_hash	1	231	2	0	0	0	0	0	0	0	0
rspack_hook	1	39	0	0	0	0	0	3	0	0	0
rspack_ids	9	1566	25	16	0	0	2	0	21	13	0
rspack_plugin_css_chunking	1	439	0	0	0	0	0	0	1	0	0
rspack_plugin_real_content_hash	2	488	9	5	0	0	0	0	6	0	2
rspack_plugin_swc_js_minimizer	1	459	7	2	0	2	0	0	4	7	0
rspack_plugin_lightning_css_minimizer	1	305	4	0	0	3	0	0	4	0	1
rspack_watcher	12	2168	31	9	1	10	9	0	8	1	13
rspack_util	24	2471	10	1	4	0	4	0	9	13	0
```

## How to use this

- Treat these counts as **triage signals**, not direct proof of runtime hotspots.
- Use with runtime traces (`RSPACK_PROFILE`, react-10k phase timings) to prioritize work.
- Start optimization design from crates that are high in both runtime evidence and static signal density.
