# rspack_loader_runner — Performance Opportunities

**Size**: 1,853 lines of Rust across 7 files  
**Role**: Executes the loader chain for each module — orchestrates the sequence of loaders that transform source code  
**Impact**: Medium — runs for every module, but overhead is mostly in the loaders themselves

---

## Table of Contents

1. [Loader Chain Execution Model](#1-loader-chain-execution-model)
2. [Content Conversion Overhead](#2-content-conversion-overhead)
3. [File Dependency Collection](#3-file-dependency-collection)

---

## 1. Loader Chain Execution Model

**File**: `crates/rspack_loader_runner/src/runner.rs`

The loader runner executes loaders in a pitching phase then a normal phase:

```rust
pub async fn run_loaders<Context: Send>(
    loaders: Vec<Arc<dyn Loader<Context>>>,
    resource_data: Arc<ResourceData>,
    plugin: Option<Arc<dyn LoaderRunnerPlugin<Context = Context>>>,
    context: Context,
    fs: Arc<dyn ReadableFileSystem>,
) -> Result<LoaderResult<Context>> {
    let mut loader_context = create_loader_context(loader_items, resource_data, plugin, context);
    // Pitch phase: iterate loaders forward
    // Normal phase: iterate loaders backward
    // Process resource if no loader handled it
}
```

Each loader is called individually with an `async` boundary. For modules with a single builtin loader (which is the common case for plain JS), the overhead of the loader framework is:
- Creating `LoaderContext` (allocating multiple HashSets for dependencies)
- Virtual dispatch to the loader
- Converting between Content types

**Opportunity**:
1. **Fast path for single builtin loader**: When there's only one builtin loader (e.g., `builtin:swc-loader`), skip the full loader framework and call the loader directly.
2. **Pre-allocate dependency sets**: Use `SmallVec` or `ArrayVec` for `file_dependencies`, `context_dependencies`, `missing_dependencies` since most modules have few dependencies.

**Impact**: Low. The loader framework overhead is small relative to actual loader execution.

**Estimated Gain**: 1-2% of make phase

---

## 2. Content Conversion Overhead

**File**: `crates/rspack_loader_runner/src/content.rs`

Content can be string or buffer:
```rust
pub enum Content {
    String(String),
    Buffer(Vec<u8>),
}
```

`into_string_lossy()` is called frequently to convert content to a string for processing. If the content is already a string, this is zero-copy. But for buffer content, it involves a UTF-8 conversion.

**Opportunity**: Minor. Most JavaScript content starts as strings.

**Estimated Gain**: Negligible

---

## 3. File Dependency Collection

The `LoaderContext` creates multiple `HashSet<PathBuf>` for tracking dependencies:

```rust
LoaderContext {
    file_dependencies: Default::default(),     // HashSet<PathBuf>
    context_dependencies: Default::default(),
    missing_dependencies: Default::default(),
    build_dependencies: Default::default(),
    // ...
}
```

At 10K modules, that's 40K `HashSet` allocations. Most modules have exactly 1 file dependency (the module file itself) and 0 context/missing/build dependencies.

**Opportunity**:
1. Use `SmallVec<[PathBuf; 1]>` or similar for the common case of 1 file dependency.
2. Use `Option<HashSet>` — allocate the HashSet only when there are multiple dependencies.

**Impact**: Low. HashSet default allocation is small.

**Estimated Gain**: <1%

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Fast path for single builtin loader | 1-2% of make phase | Medium |
| 2 | SmallVec for dependency collections | <1% | Low |
