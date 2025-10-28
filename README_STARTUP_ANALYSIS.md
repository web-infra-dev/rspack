# Rspack Startup Bootstrap Analysis - Documentation Index

This directory contains comprehensive documentation of how rspack renders the "run startup" bootstrap code at the end of bundles.

## Quick Start

Choose your starting point based on your needs:

### For Quick Understanding (5 minutes)
Start with: **ANALYSIS_SUMMARY.md** (340 lines)
- Executive overview of the three startup paths
- Critical decision points with file locations
- Key insights and configuration examples
- Quick reference tables

### For Visual Learners (10 minutes)
Use: **STARTUP_DECISION_TREE.txt** (440 lines)
- ASCII flowchart of complete decision sequence
- Step-by-step process visualization
- Runtime template definitions (JavaScript)
- Final output paths summary with examples

### For Complete Understanding (30 minutes)
Read: **STARTUP_BOOTSTRAP_ANALYSIS.md** (481 lines)
- Comprehensive architectural overview
- 5-level detailed decision tree
- Runtime requirement flow explanation
- Generated output examples for each path
- Mutual exclusivity and design insights

### For Developer Reference (ongoing)
Reference: **CODE_SNIPPETS_REFERENCE.md** (474 lines)
- Actual Rust code from the codebase
- Line numbers for each code section
- JavaScript output examples
- Complete file locations table

## What This Analysis Covers

This analysis examines how rspack decides which startup method to use for bundle execution:

1. **STARTUP Path** (`__webpack_require__.x()`)
   - Synchronous function wrapper
   - Used when entry modules have dependent chunks
   - Most common pattern

2. **STARTUP_ENTRYPOINT Path** (`__webpack_require__.X()`)
   - Async Module Federation startup
   - Returns Promise for concurrent chunk loading
   - Enabled with `experiments.mfAsyncStartup = true`

3. **ON_CHUNKS_LOADED Path** (`__webpack_require__.O()`)
   - Deferred/passive chunk loading
   - Two-phase execution (register + execute)
   - Default for array-push-callback format

## Key Files Analyzed

### Core Logic
- `/crates/rspack_plugin_javascript/src/plugin/mod.rs` (lines 227-552)
  - Main `render_bootstrap()` function
  - Contains all decision logic

### Helper Functions
- `/crates/rspack_plugin_runtime/src/helpers.rs` (lines 181-273)
  - `generate_entry_startup()` function
  - Generates initial entry startup code

### Runtime Requirement Plugins
- `/crates/rspack_plugin_runtime/src/array_push_callback_chunk_format.rs` (lines 56-60)
  - Sets STARTUP_ENTRYPOINT vs ON_CHUNKS_LOADED
  - Determines async vs passive mode

- `/crates/rspack_plugin_runtime/src/startup_chunk_dependencies.rs` (lines 25-47)
  - Sets STARTUP flag when chunks have dependencies
  - Determines wrapping vs inlining

### Definitions
- `/crates/rspack_core/src/runtime_globals.rs`
  - RuntimeGlobals enum definitions
  - JavaScript mapping names

### Templates
- `startup_entrypoint.ejs` - Sync MF startup
- `startup_entrypoint_with_async.ejs` - Async MF startup
- `on_chunk_loaded.ejs` - Deferred callback handler
- `startup_chunk_dependencies.ejs` - STARTUP wrapper

## The Decision Tree (Simplified)

```
render_bootstrap()
│
├─ STARTUP_NO_DEFAULT set?
│  ├─ NO (normal) → Process entry modules
│  │   ├─ Has entries? → Build startup logic
│  │   │   ├─ Check 8 inline bailouts
│  │   │   ├─ Generate module execution code (3 cases)
│  │   │   └─ Wrap or inline?
│  │   │       ├─ STARTUP required? → Wrap in function (A1a)
│  │   │       └─ NO → Inline code (A1b)
│  │   └─ No entries? → Empty STARTUP if needed
│  │
│  └─ YES → Skip entry processing
│      ├─ STARTUP_ENTRYPOINT required? → .X() path (B)
│      └─ STARTUP required? → .x() path or empty (C)
```

## Configuration Affecting Startup

```javascript
// rspack.config.js
{
  experiments: {
    mfAsyncStartup: true  // Controls startup path selection
  }
}
```

When `mfAsyncStartup = true`:
- Uses STARTUP_ENTRYPOINT (.X) with Promise-based async loading
- Enables concurrent chunk resolution

When `mfAsyncStartup = false`:
- Uses ON_CHUNKS_LOADED (.O) with deferred callback pattern
- Sequential chunk loading via two-phase execution

## Generated Code Examples

### Simple Entry (Inlined)
```javascript
var __webpack_exports__ = __webpack_require__(0);
```

### Entry with Dependencies (Wrapped)
```javascript
__webpack_require__.x = function() {
    var __webpack_exports__ = {};
    __webpack_exports__ = __webpack_require__.O(
        __webpack_exports__,
        [1, 2],  // dependent chunks
        function() { return __webpack_require__(0); }
    );
    return __webpack_exports__;
};
var __webpack_exports__ = __webpack_require__.x();
```

### Async MF (Promise-based)
```javascript
__webpack_require__.X = function(result, chunkIds, fn) {
    // async version with Promise.all()
    return Promise.all(chunkIds.map(__webpack_require__.e, __webpack_require__))
        .then(function() {
            var r = fn();
            return r === undefined ? result : r;
        });
}
var __webpack_exports__ = __webpack_require__.X();  // Returns Promise
```

## Key Insights

### 1. Mutual Exclusivity
Only ONE startup method (.x, .X, or .O) per chunk. They cannot coexist.

### 2. Inline Optimization
Code is inlined directly when:
- STARTUP flag not required
- All 8 bailout conditions are false
- Saves function definition overhead

### 3. Two-Phase ON_CHUNKS_LOADED
- Phase 1: Register callback with chunk IDs
- Phase 2: Execute deferred handlers when chunks load

### 4. Async Module Federation
- Returns Promise for truly concurrent chunk loading
- Allows parallel dependency resolution
- Differs from sequential STARTUP wrapper

### 5. The "passive" Flag
- `passive=true` → ON_CHUNKS_LOADED (.O)
- `passive=false` → STARTUP_ENTRYPOINT (.X)
- Determined by: `\!experiments.mf_async_startup`

## 8 Inline Bailout Conditions

Code cannot be inlined if any of these are true:
1. Module factories required
2. Module cache required
3. Module execution intercepted
4. Entry depends on other chunks
5. Entry referenced by other modules
6. No top-level declarations metadata
7. Hook bailout detected
8. Entry requires 'module' global

## Runtime Global Mappings

| Constant | Bit | JavaScript | Purpose |
|----------|-----|------------|---------|
| STARTUP_ENTRYPOINT | 1<<34 | `__webpack_require__.X` | Async MF startup |
| STARTUP | 1<<42 | `__webpack_require__.x` | Sync startup wrapper |
| ON_CHUNKS_LOADED | 1<<15 | `__webpack_require__.O` | Deferred callbacks |
| EXPORTS | 1<<44 | `__webpack_exports__` | Entry exports |
| STARTUP_NO_DEFAULT | 1<<40 | - | Marker flag |

## File Organization

```
rspack/
├── ANALYSIS_SUMMARY.md (START HERE)
│   └─ Quick overview and reference tables
├── STARTUP_DECISION_TREE.txt
│   └─ Visual flowchart and sequence diagrams
├── STARTUP_BOOTSTRAP_ANALYSIS.md
│   └─ Complete technical deep-dive
├── CODE_SNIPPETS_REFERENCE.md
│   └─ Code from codebase with annotations
└── README_STARTUP_ANALYSIS.md (this file)
    └─ Navigation guide and quick reference
```

## Code Locations Index

| Component | File | Lines |
|-----------|------|-------|
| render_bootstrap | rspack_plugin_javascript/src/plugin/mod.rs | 227-552 |
| generate_entry_startup | rspack_plugin_runtime/src/helpers.rs | 181-273 |
| STARTUP_ENTRYPOINT assignment | array_push_callback_chunk_format.rs | 56-60 |
| STARTUP assignment | startup_chunk_dependencies.rs | 25-47 |
| Runtime globals | rspack_core/src/runtime_globals.rs | 185, 201, 320, 328 |

## Next Steps

1. **Quick Start**: Read ANALYSIS_SUMMARY.md (5 min)
2. **Visual Understanding**: Review STARTUP_DECISION_TREE.txt (10 min)
3. **Deep Dive**: Study STARTUP_BOOTSTRAP_ANALYSIS.md (30 min)
4. **Code Reference**: Use CODE_SNIPPETS_REFERENCE.md while reading source

## Questions This Analysis Answers

- How does rspack decide which startup method to use?
- What are the three different startup execution paths?
- When is code inlined vs wrapped in a function?
- How does Module Federation async startup work?
- What does "var __webpack_exports__ = __webpack_require__.X();" mean?
- How does ON_CHUNKS_LOADED work with two-phase execution?
- What triggers the STARTUP vs STARTUP_ENTRYPOINT decision?
- What are the 8 inline bailout conditions?
- How do runtime globals map to JavaScript functions?

## Related Configuration Files

- `examples/basic/rspack.config.cjs` - Example with `mfAsyncStartup: true`
- `packages/rspack/src/config/types.ts` - Configuration type definitions
- `crates/rspack_core/src/options/experiments/mod.rs` - Experiment definitions

---

**Generated**: October 27, 2025
**Branch**: feature/async-startup-runtime-promise
**Total Lines of Analysis**: 1,295 lines across 4 documents
