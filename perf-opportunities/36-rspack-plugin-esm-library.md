# rspack_plugin_esm_library — Performance Opportunities

**Size**: 4,679 lines across 9 files  
**Role**: ESM output format — generates ES module output with proper import/export statements, linking, and tree-shakeable output  
**Impact**: Only relevant for library builds (not application builds like react-10k), but can be significant for monorepo library compilation

---

## Architecture

The ESM library plugin has a complex rendering pipeline:

```
1. optimize_chunks     — Restructure chunks for ESM output
2. link                — Resolve import/export chains across modules (2,376 lines!)
3. render              — Generate ESM output with imports/exports (844 lines)
4. chunk_link          — Per-chunk linking context
5. preserve_modules    — Support for preserveModules option (Rslib)
```

### The `link.rs` Module (2,376 lines)

This is the largest single file in the plugin. It handles:
- Tracing export chains through re-exports
- Resolving circular dependencies
- Computing external interop patterns
- Building a symbol reference graph

The linking uses `rayon::par_iter` for parallel processing:
```rust
rayon::{iter::Either, prelude::*};
```

### Key Operations

1. **Module re-parsing**: Like ConcatenatedModule, the ESM library plugin re-parses module sources using SWC:
```rust
use swc_core::ecma::parser::parse_file_as_module;
```

This is another instance of the **double-parsing problem** identified in `26-deep-dive-concatenated-module.md`.

2. **Export chain resolution**: For each module, traces all export chains to their final targets. With barrel files (common in libraries), this can be deeply nested.

3. **Symbol name collision avoidance**: Similar to ConcatenatedModule, generates unique names across all linked modules.

---

## Performance Concerns (Library Builds Only)

### 1. Re-parsing of All Modules

Every module in the library is re-parsed during the linking phase to analyze its imports/exports. For a library with 1000 modules, this is 1000 unnecessary re-parses.

**Same fix as ConcatenatedModule**: Cache the AST from the build phase.

### 2. Export Chain Resolution is O(n × chain_depth)

For barrel files like:
```js
// index.js
export * from './Button';
export * from './Input';
export * from './Select';
// ... 100 more
```

The plugin traces each export through the chain. With nested barrel files, this becomes O(n × d) where d is the nesting depth.

### 3. `preserve_modules` Mode Creates Many Chunks

When `preserveModules: true` (used by Rslib), each module becomes its own chunk. At 1000 library modules, this creates 1000 chunks, and all per-chunk operations (hashing, rendering, emit) scale accordingly.

---

## Impact

For the react-10k benchmark (application build), this plugin is **not active** and has zero impact.

For library builds (Rslib), this plugin can be significant:
- Library with 1000 modules + preserveModules: ~500ms-2s additional overhead
- Main bottleneck: re-parsing and export chain resolution

| # | Opportunity | Impact (Library Builds) | Effort |
|---|-----------|------------------------|--------|
| 1 | Cache AST from build phase | 20-40% of link time | High |
| 2 | Optimize barrel file export resolution | 10-20% of link time | Medium |
| 3 | Batch chunk operations for preserveModules | 10-30% of render time | Medium |
