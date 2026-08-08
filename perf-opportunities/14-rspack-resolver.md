# rspack_resolver — Performance Opportunities

**Size**: ~1,500 lines in `crates/rspack_core/src/resolver/` (wrapper around `rspack_resolver` crate)  
**Role**: Module resolution — resolves import paths to file system locations, handles aliases, extensions, package.json exports  
**Impact**: High — resolution runs for every dependency during the make phase (50K+ resolutions for react-10k)

---

## Table of Contents

1. [Resolver Cache Strategy](#1-resolver-cache-strategy)
2. [Resolver Factory DashMap Lookups](#2-resolver-factory-dashmap-lookups)
3. [Options Merging Per Resolution](#3-options-merging-per-resolution)
4. [File System Calls](#4-file-system-calls)

---

## 1. Resolver Cache Strategy

**File**: `crates/rspack_core/src/resolver/resolver_impl.rs`

The `rspack_resolver` (the external crate) has an internal cache for resolved paths. However, cache hits depend on the resolver options and context.

The resolver factory caches resolvers by `ResolveOptionsWithDependencyType`:

```rust
pub struct ResolverFactory {
    base_options: Resolve,
    resolver: Resolver,
    resolvers: DashMap<ResolveOptionsWithDependencyType, Arc<Resolver>, BuildHasherDefault<FxHasher>>,
}
```

**Opportunity**:
1. **Pre-warm resolver cache**: For the react-10k case, many modules resolve `react`, `react-dom`, and common utilities. Pre-resolve common dependencies at startup.
2. **Resolution result caching**: Cache the full resolution result (not just the resolver) at the compilation level. The same `(context, request)` pair always resolves to the same file.
3. **Negative caching**: Cache failed resolutions to avoid repeated file system probing for non-existent paths.

**Impact**: High. Resolution involves file system operations which are the slowest part of module building.

**Estimated Gain**: 10-30% of resolution time (may already be handled by `rspack_resolver` internal cache)

---

## 2. Resolver Factory DashMap Lookups

```rust
pub fn get(&self, options: ResolveOptionsWithDependencyType) -> Arc<Resolver> {
    if let Some(r) = self.resolvers.get(&options) {
        r.clone()
    } else {
        // Create and cache new resolver
    }
}
```

Each resolution goes through a `DashMap::get()` lookup to find the appropriate resolver. With many dependency categories (ESM import, CJS require, URL, etc.), there are several resolver variants.

**Opportunity**:
1. **Cache the common resolvers**: The most common resolvers (ESM, CJS) could be cached in direct fields instead of the DashMap
2. **Thread-local resolver cache**: Use thread-local caching to avoid DashMap lock overhead

**Impact**: Low. The DashMap lookup is fast and there are few resolver variants.

**Estimated Gain**: <1%

---

## 3. Options Merging Per Resolution

When a module has custom resolve options (from `Rule.resolve`), the base options are merged:

```rust
let merged_options = match &options.resolve_options {
    Some(o) => base_options.merge(*o.clone()),
    None => base_options,
};
```

The `merge` operation clones and combines resolve options, which includes vectors of extensions, aliases, and module directories.

**Opportunity**: 
1. **Pre-compute merged options**: Most modules use the same rule-level resolve options. Cache the merged result.
2. **Reference-counted options**: Use `Arc<Resolve>` to share unchanged options without cloning.

**Impact**: Low. Most modules don't have custom resolve options.

**Estimated Gain**: <1%

---

## 4. File System Calls

Resolution involves checking for file existence across many possible paths:
- Try each extension (`.js`, `.jsx`, `.ts`, `.tsx`, `.json`, etc.)
- Try `index` files in directories
- Check `package.json` exports/main
- Follow symlinks

Each check involves a file system call. For 10K modules with ~5 dependencies each, that's 50K resolutions × ~10 file system probes each = 500K file system calls.

**Opportunity**:
1. **Directory listing cache**: Cache `readdir` results to avoid repeated directory traversals
2. **Negative result cache**: Cache "file does not exist" results to skip future probes
3. **Batch file system operations**: Check multiple files in a single syscall using batch APIs

Note: `rspack_resolver` already implements some of these optimizations internally.

**Impact**: Medium. File system operations dominate resolution time.

**Estimated Gain**: 10-20% of resolution time (if not already cached by rspack_resolver)

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Compilation-level resolution result caching | 10-30% of resolution | Medium |
| 2 | Directory listing cache | 10-20% of resolution | Medium |
| 3 | Pre-warm common dependency resolutions | 5-10% of resolution | Low |
