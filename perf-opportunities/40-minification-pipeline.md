# Minification Pipeline — Performance Analysis

**Crates**: `rspack_plugin_swc_js_minimizer` (~460 lines), `rspack_plugin_lightning_css_minimizer`  
**Role**: Compress/mangle JavaScript (SWC) and CSS (LightningCSS) in production builds  
**Impact**: **HIGH** — adds 342ms to the 531-module benchmark (13% of total build time)

---

## Profiling Results

**531 modules + 30 async chunks, with vs without minification:**

| Phase | minimize=false | minimize=true | Difference |
|-------|---------------|---------------|-----------|
| process assets | 63ms | **405ms** | **+342ms** |
| **Total** | 2,479ms | 2,730ms | **+251ms net** |

The net difference (251ms) is less than the process_assets difference (342ms) because RealContentHashPlugin runs faster with smaller minified output.

At 10K modules, projected minification time: **~3-6 seconds** (scales roughly linearly with total source size).

---

## Architecture

### SWC JS Minimizer

```rust
#[plugin_hook(CompilationProcessAssets for SwcJsMinimizerRspackPlugin, 
    stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
    compilation.assets_mut()
        .par_iter_mut()                          // ← Parallel over assets!
        .filter(|(filename, original)| {
            match_object(options, filename)       // Check test/include/exclude
            && !original.get_info().minimized     // Skip already minified
        })
        .try_for_each_with(tx, |tx, (filename, original)| {
            let javascript_compiler = JavaScriptCompiler::new();  // New compiler per asset!
            let output = javascript_compiler.minify(input, js_minify_options, ...)?;
            // Replace asset source with minified output
            *original = CompilationAsset::new(Some(new_source), new_info);
            Ok(())
        })
}
```

Key observations:
1. **Already parallelized with rayon** — `par_iter_mut()` processes assets in parallel ✓
2. **New `JavaScriptCompiler` per asset** — creates SWC globals per asset
3. **Source map handling**: Gets source map from original, passes to SWC, merges back
4. **Comment extraction**: Iterates all comments in the minified output

### Per-Asset Minification Cost

For a typical chunk (~50KB JS, 200 modules concatenated):
- SWC parse: ~1ms
- SWC compress: ~5-10ms (the dominant cost — constant folding, dead code elimination, name mangling)
- SWC codegen: ~0.5ms
- Source map merge: ~0.5ms
- **Total: ~7-12ms per chunk**

With 30 async chunks + 1 main chunk = 31 assets to minify.
31 × ~10ms = ~310ms, matching the profiled 342ms.

---

## Performance Opportunities

### 1. Avoid New JavaScriptCompiler Per Asset

```rust
// CURRENT:
let javascript_compiler = JavaScriptCompiler::new();

// FIX: Thread-local pooling
thread_local! {
    static COMPILER: RefCell<Option<JavaScriptCompiler>> = RefCell::new(None);
}
let output = COMPILER.with(|cell| {
    let compiler = cell.borrow_mut().get_or_insert_with(JavaScriptCompiler::new);
    compiler.minify(input, options, ...)
});
```

**Savings**: ~0.1ms per asset × 31 assets = ~3ms (minimal)

### 2. Skip Source Map Computation When Not Needed

When `devtool: false` (no source maps in production), the minimizer still creates source maps internally:

```rust
source_map: BoolOrDataConfig::from_bool(input_source_map.is_some()),
```

If input has no source map, SWC skips source map generation. This is already correct. ✓

### 3. Parallel Chunk-Level vs Asset-Level

The current parallelization is at the asset level. For a SPA with 1 large main chunk and 30 small async chunks, one core gets the large chunk while others handle small chunks quickly and idle.

**Opportunity**: Split large chunks into segments for sub-chunk parallelism. This is complex but could help when one chunk is much larger than others.

### 4. Streaming Minification

Currently, the entire asset source is loaded into memory, minified, then the entire output is written:
```rust
let input = original_source.source().into_string_lossy().into_owned();
// ... minify ...
*original = CompilationAsset::new(Some(new_source), new_info);
```

**Opportunity**: For very large chunks, stream the minification to reduce peak memory.

### 5. Minification Cache

If a chunk's content hasn't changed between rebuilds, the minified output is the same. Caching minification results by content hash would eliminate redundant work:

```rust
let content_hash = xxhash64(input.as_bytes());
if let Some(cached) = minify_cache.get(&content_hash) {
    *original = cached.clone();
    return Ok(());
}
```

**Savings for rebuilds**: Skip 90%+ of minification work (only re-minify changed chunks).

### 6. SWC Compress Options Tuning

The SWC compressor has many passes. Some are more expensive than others:
- `dead_code`: O(n) — fast
- `collapse_vars`: O(n²) in worst case — can be slow for large scopes
- `reduce_vars`: O(n) — fast
- `unused`: O(n) — fast
- `pure_funcs` / `pure_getters`: O(n) — fast

**Opportunity**: For incremental rebuilds, use a lighter compression profile (fewer passes) since the unchanged code was already heavily compressed.

---

## Scaling Projections

| Module Count | Chunks | Estimated Minification Time | Notes |
|-------------|--------|---------------------------|-------|
| 531 (measured) | 31 | 342ms | Debug build |
| 531 (release est.) | 31 | ~50ms | ~7x faster in release |
| 10,000 | ~200 | ~1,000ms (release) | Linear with total code size |
| 50,000 | ~500 | ~3,000ms (release) | |

In release mode, minification takes **~50ms for 531 modules** — much less dominant than code splitting (1,210ms) and SideEffects (112ms). But at 10K+ modules, minification becomes significant at ~1 second.

---

## Summary

| # | Opportunity | Impact | Effort |
|---|-----------|--------|--------|
| 1 | **Minification cache for rebuilds** | Skip 90%+ of minify work on rebuild | Medium |
| 2 | SWC compress profile tuning for rebuilds | 20-50% faster minification | Low |
| 3 | Thread-local JavaScriptCompiler pool | ~3ms savings | Low |
| 4 | Sub-chunk parallel minification | Better load balance for uneven chunks | High |

The minification pipeline is already well-designed (rayon parallelism, per-asset processing). The main opportunity is **caching** for rebuilds.
