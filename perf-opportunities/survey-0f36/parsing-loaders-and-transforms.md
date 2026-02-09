# Parsing, Loaders, and Transforms

This document covers the loader pipeline and JS/CSS transform stages.

## Where the Work Happens

- Loader runner: `crates/rspack_loader_runner/src/runner.rs`
- Loader content handling: `crates/rspack_loader_runner/src/content.rs`
- SWC loader: `crates/rspack_loader_swc/**`
- JavaScript compiler: `crates/rspack_javascript_compiler/**`
- JS plugin pipeline: `crates/rspack_plugin_javascript/**`
- LightningCSS loader: `crates/rspack_loader_lightningcss/**`

## Hotspot Evidence

Perf shows `core::str::lossy::Utf8Chunks::next`, plus SWC minifier and JS
inner-graph processing (`rspack_plugin_javascript::parser_plugin::inner_graph`)
in the extended samples. This matches multiple `String::from_utf8_lossy`
conversions in loader content handling:

- `Content::into_string_lossy` in `rspack_loader_runner/src/content.rs`
- `Content` debug output uses lossy conversions for truncation

Extended samples also show `simdutf8::validate_utf8_basic` and `hstr::AtomStore`
insertions, indicating heavy UTF‑8 validation + atom interning during parsing.

## Optimization Opportunities

### 1) Avoid lossy string conversions in hot paths

`Content` and loaders frequently convert buffers into strings. Many transforms
can operate on `&[u8]` or `Cow<str>` without forcing lossy conversions:

- Add a zero-copy `Content::as_str()` for UTF‑8‑validated buffers.
- Cache a `String` representation inside `Content` to avoid repeated conversions.
- Keep debug formatting behind trace flags to avoid unnecessary conversions.
- Avoid repeated UTF‑8 validation by caching validation results for unchanged inputs.
- Consider sharing atom stores or interning scopes to reduce `AtomStore::insert_entry`.

### 2) Reduce loader pipeline allocations

`run_loaders_impl` builds `LoaderContext` with multiple `HashSet`/`Vec` fields.
When processing many modules, these allocations add up:

- Reuse loader contexts for identical module rules.
- Pool `HashSet` buffers (especially dependency sets).
- Avoid cloning loader vectors; use `Arc<[LoaderItem]>` when possible.

### 3) Cache SWC configurations and AST transforms

`rspack_loader_swc` + `rspack_javascript_compiler` frequently rebuild SWC
options. Opportunities:

- Cache normalized SWC configs per module type.
- Reuse AST arenas for repeated transforms.
- Avoid serialization/deserialization between JS and Rust for identical options.

### 4) Parallelize loader chains more aggressively

The loader runner already yields to JS, but full module builds can be parallel:

- Batch IO reads and parse work for independent modules.
- Run CSS/JS transforms in separate task pools with bounded concurrency.

### 5) CSS pipeline: LightningCSS batching

`rspack_loader_lightningcss` can be expensive for large CSS graphs:

- Cache parsed CSS ASTs for unchanged modules.
- Combine multiple small CSS files into a single parsing unit where possible.

### 6) Minimize per-module diagnostics overhead

Diagnostics aggregation can create many small allocations in failure-free
builds. Use lazy diagnostics creation only when errors occur.
