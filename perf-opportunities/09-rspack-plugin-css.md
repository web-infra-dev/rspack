# rspack_plugin_css — Performance Opportunities

**Size**: 3,316 lines of Rust across 14 files  
**Role**: CSS module parsing, dependency scanning, code generation, and chunk ordering  
**Impact**: Medium — affects CSS modules which are a smaller fraction of react-10k, but CSS ordering logic can be expensive

---

## Table of Contents

1. [CSS Module Ordering Algorithm](#1-css-module-ordering-algorithm)
2. [CSS Parsing Duplication](#2-css-parsing-duplication)
3. [Chunk Group Cloning in Sort](#3-chunk-group-cloning-in-sort)

---

## 1. CSS Module Ordering Algorithm

**File**: `crates/rspack_plugin_css/src/plugin/mod.rs`

The CSS ordering uses `get_modules_in_order` which sorts modules per chunk group:

```rust
let mut modules_by_chunk_group = chunk.groups().iter()
    .map(|group| compilation.build_chunk_graph_artifact.chunk_group_by_ukey.expect_get(group))
    .map(|chunk_group| {
        let mut indexed_modules = modules_list.clone()  // CLONE of entire modules list per group
            .into_iter()
            .filter_map(|module| {
                let index = chunk_group.module_post_order_indices.get(&module.identifier())?;
                Some((*index, module))
            })
            .collect::<Vec<_>>();
        indexed_modules.sort_by_key(|(index, _)| *index);
        // ...
    })
    .collect::<Vec<_>>();
```

For each chunk group, the entire module list is cloned and filtered. With many chunk groups per chunk, this creates significant allocation.

**Opportunity**:
1. **Avoid cloning the module list**: Use references or indices instead of cloning
2. **Pre-compute CSS module order**: Compute once during seal and cache
3. **Use index-based sorting**: Sort module indices directly rather than (index, module) tuples

**Impact**: Medium for CSS-heavy projects, Low for react-10k (which is primarily JS).

**Estimated Gain**: 10-30% of CSS ordering time

---

## 2. CSS Parsing Duplication

CSS modules use `lightningcss` for parsing. The parser and generator follows a similar pattern to JS — parse in build, generate in code generation. Source maps are handled similarly to JS.

**Opportunity**: Same as the JS plugin — ensure source map processing is lazy and avoid unnecessary re-parsing.

**Estimated Gain**: 2-5% of CSS processing time

---

## 3. Chunk Group Cloning in Sort

The `modules_by_chunk_group` computation iterates all chunk groups and creates sorted lists. The final merge uses a selection algorithm that finds the module with the best ordering across all groups.

**Opportunity**: Use a merge-sort style algorithm instead of the selection-based approach for better cache performance.

**Estimated Gain**: Minor

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Avoid module list cloning in CSS ordering | 10-30% of CSS ordering | Low |
| 2 | Lazy source map processing | 2-5% of CSS processing | Medium |
