# Rspack Startup Bootstrap Deep Dive - Summary Report

## Overview

This analysis covers the complete decision tree and logic flow for rspack's "run startup" bootstrap code rendering, specifically analyzing how the system decides between three mutually exclusive startup execution paths.

## Analysis Documents Generated

### 1. STARTUP_BOOTSTRAP_ANALYSIS.md (Main Document)
**Purpose**: Comprehensive technical deep-dive with full architecture overview

**Contents**:
- Executive summary of the three startup paths
- Complete architectural overview with ASCII diagrams
- Detailed decision tree at 5 levels
- Runtime requirement flow explanation
- Helper function analysis (generate_entry_startup)
- Generated output examples for each path
- Key insights and conclusions

**Key Sections**:
- Architecture Overview showing complete flow
- Complete Decision Tree with all branches
- Runtime Requirement Flow showing where STARTUP_ENTRYPOINT and STARTUP come from
- Critical Code Locations table with line numbers
- Decision Flowchart in ASCII format
- Key Insights section explaining mutual exclusivity, inlining, double calls, and async mode

### 2. STARTUP_DECISION_TREE.txt (Visual Guide)
**Purpose**: Detailed ASCII flowchart showing the exact decision sequence

**Contents**:
- Entry point and primary decision
- All decision branches with outputs
- Parallel runtime requirement assignment
- Helper function logic
- Runtime template definitions (JavaScript code)
- Final output paths summary

**Visual Structure**:
Shows the complete flow from `render_bootstrap()` entry through all decision points, with generated JavaScript code examples for each path.

### 3. CODE_SNIPPETS_REFERENCE.md (Developer Reference)
**Purpose**: Actual code snippets from the rspack codebase with explanations

**Contents**:
- Path selection logic (actual Rust code)
- Path A1a: STARTUP Wrapper (with generated output)
- Path A1b: STARTUP Inline code
- Path B: STARTUP_ENTRYPOINT (async MF)
- Path C: Minimal STARTUP
- Module execution logic (3 cases)
- Runtime requirement assignment
- Helper function implementation
- Runtime global definitions
- Template definitions (JavaScript)
- Inline bailout checks
- Complete file locations table

## The Decision Tree Summary

### Top Level: STARTUP_NO_DEFAULT

The system starts with a single flag check:

```
STARTUP_NO_DEFAULT not set?
├─ YES (Normal case) -> Process entry modules
└─ NO -> Skip to simplified paths
```

### Level 2: Entry Modules Exist

If in normal case:

```
Has entry modules?
├─ YES -> Build startup logic (most complex)
└─ NO -> Define empty STARTUP function (if needed)
```

### Level 3: Build Startup Logic (Complex Path)

For entries WITH modules:

1. **Check inline bailouts** (8 conditions that disable inlining)
2. **Build module execution code** (3 cases: dependent chunks, use_require, direct)
3. **Post-process** with ON_CHUNKS_LOADED if needed
4. **Decide wrapping strategy** (STARTUP wrapper vs inline)

### Level 4 & 5: Final Paths

After all logic:

```
Decide final output:
├─ Path A1a: Wrap in STARTUP function (.x)
├─ Path A1b: Inline code directly (no wrapper)
├─ Path B: Use STARTUP_ENTRYPOINT (.X) for async MF
└─ Path C: Empty STARTUP function (stub)
```

## The Three Startup Methods

### 1. STARTUP Path (.x) - Synchronous

**Function**: `__webpack_require__.x()`

**Use Case**: Entry modules with dependent chunks

**Generated Output**:
```javascript
__webpack_require__.x = function() {
    var __webpack_exports__ = {};
    // execute entry modules with chunk loading
    return __webpack_exports__;
};
var __webpack_exports__ = __webpack_require__.x();
```

**Key Characteristic**: Wraps logic in a function, called synchronously

### 2. STARTUP_ENTRYPOINT Path (.X) - Async Module Federation

**Function**: `__webpack_require__.X()`

**Use Case**: Module Federation with `experiments.mfAsyncStartup = true`

**Generated Output**:
```javascript
var __webpack_exports__ = __webpack_require__.X();
```

**Key Characteristic**: Can return Promise if async template used, allows concurrent chunk loading

**Two Variants**:
- **Sync version**: Direct execution, same basic flow
- **Async version**: Returns `Promise.all()` for concurrent chunk loading

### 3. ON_CHUNKS_LOADED Path (.O) - Deferred/Passive

**Function**: `__webpack_require__.O()`

**Use Case**: Array-push-callback format when `experiments.mfAsyncStartup = false`

**Generated Output**:
```javascript
__webpack_require__.O(0, [chunk_ids], function() {
    return __webpack_exec__(module_id);
});
var __webpack_exports__ = __webpack_require__.O();
```

**Key Characteristic**: Two-phase execution (register callback, then execute deferred queue)

## Critical Decision Points

### Decision 1: STARTUP_NO_DEFAULT Flag
**File**: rspack_plugin_javascript/src/plugin/mod.rs:325
**Impact**: Controls whether entry modules are processed or simpler paths are taken

### Decision 2: Entry Module Existence
**File**: rspack_plugin_javascript/src/plugin/mod.rs:326
**Impact**: Determines if startup logic needs to be built

### Decision 3: Inline Bailouts (8 conditions)
**File**: rspack_plugin_javascript/src/plugin/mod.rs:247-412
**Impact**: Determines if code can be inlined or must be wrapped

**Bailout Conditions**:
1. Module factories used
2. Module cache used
3. Module execution intercepted
4. Entry depends on other chunks
5. Entry referenced by other modules
6. No top-level declarations metadata
7. Hook bailout detected
8. Entry requires 'module' global

### Decision 4: Module Execution Method
**File**: rspack_plugin_javascript/src/plugin/mod.rs:421-483
**Three Cases**:
1. Entry has dependent chunks -> Use ON_CHUNKS_LOADED
2. use_require flag set -> Direct __webpack_require__ call
3. Direct execution -> Call __webpack_modules__ directly with parameter matching

### Decision 5: STARTUP Required?
**File**: rspack_plugin_javascript/src/plugin/mod.rs:495
**Impact**: Determines if code is wrapped in function or inlined

### Decision 6: STARTUP_ENTRYPOINT vs ON_CHUNKS_LOADED
**File**: rspack_plugin_runtime/src/array_push_callback_chunk_format.rs:56-60
**Determining Factor**: `experiments.mf_async_startup` configuration flag
- `true` -> STARTUP_ENTRYPOINT (async, Promise-based)
- `false` -> ON_CHUNKS_LOADED (deferred, register+execute)

### Decision 7: STARTUP Flag Set?
**File**: rspack_plugin_runtime/src/startup_chunk_dependencies.rs:38
**Determining Factor**: `has_chunk_entry_dependent_chunks`
- `true` -> STARTUP required (wrap logic in function)
- `false` -> STARTUP not required (can inline)

## Runtime Globals Reference

| Global | Bit | Symbol | JavaScript | Purpose |
|--------|-----|--------|------------|---------|
| STARTUP_ENTRYPOINT | 1<<34 | .X | `__webpack_require__.X` | Async MF startup |
| STARTUP | 1<<42 | .x | `__webpack_require__.x` | Sync startup wrapper |
| ON_CHUNKS_LOADED | 1<<15 | .O | `__webpack_require__.O` | Deferred callbacks |
| EXPORTS | 1<<44 | - | `__webpack_exports__` | Entry module exports |
| STARTUP_NO_DEFAULT | 1<<40 | - | - | Marker flag |

## Key Insights

### 1. Mutual Exclusivity
Only ONE of the three startup methods (.x, .X, or .O) is generated per chunk. They cannot coexist.

### 2. The Inline Optimization
Code can be inlined directly if:
- STARTUP flag is NOT required
- All 8 bailout conditions are false
- This saves function definition overhead

### 3. Two-Phase ON_CHUNKS_LOADED
```javascript
// Phase 1: Register
__webpack_require__.O(0, [chunks], fn);  // Returns undefined

// Phase 2: Execute
var __webpack_exports__ = __webpack_require__.O();  // Processes queue
```

### 4. Async Module Federation
When `mf_async_startup = true`:
- Uses STARTUP_ENTRYPOINT (.X) instead of ON_CHUNKS_LOADED (.O)
- Async template returns Promise for concurrent chunk loading
- Allows truly parallel dependency resolution

### 5. The "passive" Flag
In `generate_entry_startup()`:
- `passive=true` -> ON_CHUNKS_LOADED (passive/deferred)
- `passive=false` -> STARTUP_ENTRYPOINT (active/async)
- Set by: `let passive = \!mf_async_startup`

## Code Locations Summary

| Component | File | Lines | Type |
|-----------|------|-------|------|
| Main Decision Tree | rspack_plugin_javascript/src/plugin/mod.rs | 227-552 | Function |
| Entry Startup Generation | rspack_plugin_runtime/src/helpers.rs | 181-273 | Helper |
| Array-Push Format (MF decision) | rspack_plugin_runtime/src/array_push_callback_chunk_format.rs | 56-60 | Plugin |
| Startup Dependencies (STARTUP flag) | rspack_plugin_runtime/src/startup_chunk_dependencies.rs | 25-47 | Plugin |
| Runtime Global Definitions | rspack_core/src/runtime_globals.rs | 10-265 | Constants |
| STARTUP Template | rspack_plugin_runtime/src/runtime_module/runtime/startup_chunk_dependencies.ejs | - | Template |
| ON_CHUNKS_LOADED Template | rspack_plugin_runtime/src/runtime_module/runtime/on_chunk_loaded.ejs | - | Template |
| STARTUP_ENTRYPOINT (sync) | rspack_plugin_runtime/src/runtime_module/runtime/startup_entrypoint.ejs | - | Template |
| STARTUP_ENTRYPOINT (async) | rspack_plugin_runtime/src/runtime_module/runtime/startup_entrypoint_with_async.ejs | - | Template |

## Generated Code Examples

### Simple Entry (Inline)
```javascript
var __webpack_exports__ = __webpack_require__(0);
```

### Entry with Chunks (Wrapped)
```javascript
__webpack_require__.x = function() {
    var __webpack_exports__ = {};
    __webpack_exports__ = __webpack_require__.O(
        __webpack_exports__,
        [1, 2],
        function() { return __webpack_require__(0); }
    );
    return __webpack_exports__;
};
var __webpack_exports__ = __webpack_require__.x();
```

### Async MF (Promise-based)
```javascript
var __webpack_exports__ = __webpack_require__.X();
// Returns Promise if async version used
```

## File Organization

The three generated analysis documents are designed for different use cases:

1. **STARTUP_BOOTSTRAP_ANALYSIS.md** - Read this first for complete understanding
   - Best for: Learning the full system
   - Content: Architecture, detailed decision tree, explanations
   - Length: ~5000 words

2. **STARTUP_DECISION_TREE.txt** - Use for quick reference and visualization
   - Best for: Visual learners, quick lookups
   - Content: ASCII flowcharts, step-by-step decisions, generated code
   - Format: Text-based diagrams

3. **CODE_SNIPPETS_REFERENCE.md** - Use while reading the actual source code
   - Best for: Developers working on the code
   - Content: Actual Rust code with context, line numbers, explanations
   - Format: Code-focused reference

## Related Configuration

The startup system is affected by:

```javascript
// rspack.config.js
{
  experiments: {
    mfAsyncStartup: true,  // Controls STARTUP_ENTRYPOINT vs ON_CHUNKS_LOADED
  }
}
```

Example from examples/basic/rspack.config.cjs:
```javascript
experiments: {
  mfAsyncStartup: true  // Enables async Module Federation startup
}
```

## Conclusion

Rspack's startup bootstrap is a **sophisticated multi-level decision system** that optimizes module execution based on:

1. **Configuration** (mfAsyncStartup)
2. **Module graph** (dependencies, references)
3. **Runtime requirements** (which globals are needed)
4. **Optimization constraints** (can code be inlined?)

The system produces one of four possible outputs:
- **Path A1a**: STARTUP wrapper (most common with dependencies)
- **Path A1b**: Inline code (simple entries)
- **Path B**: STARTUP_ENTRYPOINT (async MF)
- **Path C**: Empty stub (compatibility)

Each path is optimized for its specific use case, balancing code size, execution efficiency, and feature support.
