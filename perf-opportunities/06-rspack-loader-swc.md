# rspack_loader_swc — Performance Opportunities

**Size**: 4,399 lines of Rust across 11 files  
**Role**: Builtin SWC-based JavaScript/TypeScript transformer — transforms modern JS/TS to target-compatible code  
**Impact**: High — runs for every JS/TS module during the build phase, one of the most expensive per-module operations

---

## Table of Contents

1. [Per-Module SWC Compiler Instantiation](#1-per-module-swc-compiler-instantiation)
2. [Source Map Processing Overhead](#2-source-map-processing-overhead)
3. [SWC Options Cloning Per Module](#3-swc-options-cloning-per-module)
4. [TypeScript Info Collection Overhead](#4-typescript-info-collection-overhead)
5. [Stack Size Workaround](#5-stack-size-workaround)

---

## 1. Per-Module SWC Compiler Instantiation

**File**: `crates/rspack_loader_swc/src/lib.rs`

For each module, a new `JavaScriptCompiler` is created:
```rust
let javascript_compiler = JavaScriptCompiler::new();
let TransformOutput { code, mut map, diagnostics } = javascript_compiler.transform(
    source, Some(filename.clone()), comments.clone(), swc_options, ...
)?;
```

While `JavaScriptCompiler::new()` is likely lightweight, the SWC transform itself involves:
- Creating source map builder
- Setting up comment handling  
- Applying all configured SWC passes
- Code generation

**Opportunity**:
1. **Pool/reuse compilers**: Use a thread-local `JavaScriptCompiler` pool to avoid repeated setup.
2. **Share comments handler**: `SingleThreadedComments` is created fresh per module but could be cleared and reused.

**Impact**: Low-Medium. SWC internals handle most reuse, but the per-module overhead adds up at 10K modules.

**Estimated Gain**: 1-3% of SWC transform time

---

## 2. Source Map Processing Overhead

```rust
if let (Some(map), Some(resource_dir)) = (map.as_mut(), resource_path.parent()) {
    map.set_sources(
        map.sources().iter().map(|source| {
            let source_path = Path::new(source);
            if source_path.is_relative() {
                source_path.absolutize_with(resource_dir.as_std_path())
                    .to_string_lossy().into_owned()
            } else {
                source.clone()
            }
        }).collect::<Vec<_>>(),
    );
}
```

After every transform, source paths in the source map are resolved to absolute paths. This involves:
- Iterating all sources in the source map
- Path parsing and absolutization for each
- String allocation for the result

Also, input source maps are parsed from JSON strings:
```rust
if let Some(pre_source_map) = loader_context.source_map().cloned()
    && let Ok(source_map) = pre_source_map.to_json() {
    swc_options.config.input_source_map = Some(InputSourceMap::Str(source_map))
}
```

This serializes the source map to JSON string, then SWC re-parses it.

**Opportunity**:
1. **Pass source maps as structured data**: Avoid the JSON serialize/deserialize round-trip by passing the source map object directly.
2. **Defer path resolution**: Resolve source map paths lazily, only when the source map is actually emitted.
3. **Skip source maps in production**: If source maps are disabled, skip all source map processing entirely.

**Impact**: Medium for projects with source maps enabled.

**Estimated Gain**: 3-8% of SWC transform time when source maps are enabled

---

## 3. SWC Options Cloning Per Module

```rust
let swc_options = {
    let mut swc_options = self.options_with_additional.swc_options.clone();
    // ... modify swc_options ...
    swc_options
};
```

The entire `swc_options` struct is cloned for each module. This includes nested config structures like `jsc`, `env`, `transform`, etc.

**Opportunity**:
1. **Copy-on-write**: Use `Cow<SwcCompilerOptions>` and only clone when modification is needed
2. **Pre-compute variants**: If there are only a few option variants (e.g., development vs production), pre-compute them and select by reference

**Impact**: Low. The clone is mostly shallow, but at 10K modules it adds up.

**Estimated Gain**: <1%

---

## 4. TypeScript Info Collection Overhead

```rust
|program, unresolved_mark| {
    if !is_typescript { return; }
    let Some(options) = &self.options_with_additional.collect_typescript_info else { return; };
    collected_ts_info = Some(collect_typescript_info(program, ...));
}
```

TypeScript info collection walks the AST an additional time for TypeScript files. This is only needed when TypeScript info collection is enabled.

**Opportunity**: Integrate TypeScript info collection into the main SWC transform pass instead of a separate walk.

**Impact**: Low. Only affects TypeScript files when the feature is enabled.

**Estimated Gain**: 1-2% of TS file transform time

---

## 5. Stack Size Workaround

```rust
#[cfg(all(debug_assertions, not(target_family = "wasm")))]
{
    stacker::maybe_grow(2 * 1024 * 1024, 4 * 1024 * 1024, inner)
}
```

In debug mode, `stacker::maybe_grow` is used to prevent stack overflow. This adds a stack check per-module.

**Opportunity**: This is already conditional on debug mode. No production impact.

**Estimated Gain**: 0%

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Avoid source map JSON round-trip | 3-8% of SWC time with source maps | Medium |
| 2 | Pool/reuse JavaScriptCompiler instances | 1-3% of SWC time | Low |
| 3 | Pre-compute SWC options variants | <1% | Low |
