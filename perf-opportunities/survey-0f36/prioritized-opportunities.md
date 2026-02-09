# Prioritized Opportunities (Summary)

This file summarizes the highest-impact opportunities observed from profiling
and code inspection. The ordering below reflects the perf evidence and expected
effort-to-impact ratio.

## 1) Reduce allocation pressure

Evidence: `mi_malloc_aligned`, `_mi_free_delayed_block`, `clear_page_erms`.

Actions:
- Pool buffers for module sources, codegen output, and diagnostics.
- Use arenas for short-lived module graph structures.
- Reuse `Vec`/`HashSet` buffers between compilation passes.

## 2) Optimize module graph overlay + exports prefetch

Evidence: `OverlayMap::get`, `ExportsInfoGetter::prefetch_exports`,
`GetSideEffectsConnectionStateCache::get`.

Actions:
- Skip overlay in full builds or add fast paths to reduce per‑lookup overhead.
- Cache prefetch exports per module; use compact bitsets for exports.
- Avoid recomputing side‑effects connection state for unchanged modules.

## 3) Identifier/path handling

Evidence: `Ustr::from`, `FileCounter::add_files`.

Actions:
- Cache interned identifiers per compilation.
- Batch file counting updates and avoid per‑file lock contention.
- Reduce path conversions (`Utf8PathBuf`/`String`) where possible.

## 4) Loader/content string conversion

Evidence: `core::str::lossy::Utf8Chunks::next`.

Actions:
- Keep loader content as bytes until explicitly needed as UTF‑8.
- Cache decoded strings or use zero‑copy `Cow<str>`.

## 5) SWC / JS plugin inner-graph work

Evidence: SWC minifier + JS inner-graph functions in extended perf list.

Actions:
- Cache SWC configs and reuse AST arenas.
- Avoid inner-graph analysis when module is side‑effect‑free and unexported.

## 6) Re-enable incremental chunk graph

Evidence: `build_chunk_graph` has incremental disabled.

Actions:
- Restore incremental build chunk graph with correctness guardrails.
- Use feature flag to measure perf impact safely.

## 7) Reduce plugin hook overhead

Evidence: plugin-heavy stages (`process_assets`, `render_manifest`).

Actions:
- Add `is_empty` fast paths for hooks.
- Batch JS↔Rust hook calls.
