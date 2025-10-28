# Deep Dive: Rspack "Run Startup" Bootstrap Code Rendering

## Executive Summary

Rspack's startup bootstrap system is a sophisticated multi-branch control flow that determines **how entry modules are executed and exported** at the end of a bundle. The system decides between three mutually exclusive execution paths based on runtime requirements and configuration flags:

1. **STARTUP** (`__webpack_require__.x()`) - Synchronous startup function
2. **STARTUP_ENTRYPOINT** (`__webpack_require__.X()`) - Async Module Federation startup (async MF)
3. **ON_CHUNKS_LOADED** (`__webpack_require__.O()`) - Deferred chunk loading with callbacks

## Architecture Overview

```
render_bootstrap()
├── Check STARTUP_NO_DEFAULT flag
│   └── If NOT set: Process entry modules and generate startup code
│       ├── Check for entry modules (chunk.has_entry_module)
│       │   ├── YES: Generate startup logic (buf2 buffer)
│       │   │   ├── Inline entry modules if possible
│       │   │   ├── Handle ON_CHUNKS_LOADED requirements
│       │   │   └── Wrap in STARTUP function or call directly
│       │   └── NO: Define empty STARTUP function
│       └── Three possible outputs:
│           ├── STARTUP path (synchronous)
│           ├── STARTUP_ENTRYPOINT path (async MF)
│           └── ON_CHUNKS_LOADED path (deferred)
│
├── STARTUP_ENTRYPOINT found: Call __webpack_require__.X()
│   └── Only in array-push-callback format with mf_async_startup=true
│
├── STARTUP found: Call __webpack_require__.x()
│   └── Default synchronous execution path
│
└── Return RenderBootstrapResult with header, startup, and inline-ability
```

## Complete Decision Tree

### Level 1: Configuration Flag Check

```rust
if !runtime_requirements.contains(RuntimeGlobals::STARTUP_NO_DEFAULT) {
    // Branch A: Normal startup (default)
} else if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
    // Branch B: Async MF startup
} else if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
    // Branch C: Empty startup handler
}
```

The key insight: **STARTUP_NO_DEFAULT** gates the entire startup system. When NOT set (the normal case), the code processes entry modules. When set, it skips to simpler branches.

### Level 2: Entry Module Existence (when STARTUP_NO_DEFAULT is NOT set)

```rust
if chunk.has_entry_module(&compilation.chunk_graph) {
    // Branch A1: Has entry modules - generate startup code
} else if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
    // Branch A2: No entry modules but STARTUP required - empty stub
}
```

### Level 3a: Generate Startup Code (Branch A1)

This is the most complex path. For each entry module, the code builds `buf2` buffer which contains the execution logic:

#### Bailout Checks (disable inline):

1. **Module factories used** - `allow_inline_startup = false`
2. **Module cache used** - `allow_inline_startup = false`
3. **Module execution intercepted** - `allow_inline_startup = false`
4. **Entry depends on other chunks** - `allow_inline_startup = false`
5. **Entry is referenced by other modules** - `allow_inline_startup = false`
6. **Entry doesn't declare top-level declarations** - `allow_inline_startup = false`
7. **Hook bailout** - `allow_inline_startup = false`
8. **Entry requires 'module' global** - `allow_inline_startup = false`

#### Module Execution Logic (3 cases):

**Case 1: Entry has dependent chunks (chunk_ids not empty)**
```rust
if !chunk_ids.is_empty() {
    buf2.push(format!(
        "{}__webpack_require__.O(undefined, {}, function() {{ return __webpack_require__({}) }});",
        if i + 1 == entries.len() { "var __webpack_exports__ = " } else { "" },
        stringify_array(&chunk_ids),
        module_id_expr
    ));
}
```
Uses `ON_CHUNKS_LOADED` to wait for chunks before executing.

**Case 2: Entry module uses require function (use_require = true)**
```rust
else if use_require {
    buf2.push(format!(
        "{}__webpack_require__({});",
        if i + 1 == entries.len() { "var __webpack_exports__ = " } else { "" },
        module_id_expr
    ));
}
```
Direct require call.

**Case 3: Direct module execution (use_require = false)**
```rust
else {
    // Requires complex parameter matching based on what globals the entry uses
    if require_scope_used {
        __webpack_modules__[moduleId](0, __webpack_exports__, __webpack_require__);
    } else if needs_exports {
        __webpack_modules__[moduleId](0, __webpack_exports__);
    } else {
        __webpack_modules__[moduleId]();
    }
}
```

#### ON_CHUNKS_LOADED Post-processing

```rust
if runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED) {
    buf2.push("__webpack_exports__ = __webpack_require__.O(__webpack_exports__);");
}
```

When ON_CHUNKS_LOADED is required, update exports with the deferred handler.

#### Final Wrapping Decision (CRITICAL):

```rust
if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
    // PATH A1a: Wrap in STARTUP function (synchronous)
    allow_inline_startup = false;
    header.push("__webpack_require__.x = function() {
        <buf2 code>
        return __webpack_exports__;
    };");
    startup.push("var __webpack_exports__ = __webpack_require__.x();");
    
} else {
    // PATH A1b: Inline startup code directly (no wrapper)
    startup.push("<buf2 code as-is>");
}
```

**KEY DISTINCTION**: 
- **PATH A1a**: When `STARTUP` is required, code goes into a function definition in the header, then called at the end
- **PATH A1b**: When `STARTUP` is not required, code executes inline directly without wrapping

### Level 3b: STARTUP_ENTRYPOINT Path (Branch B - async MF)

**ONLY reached when STARTUP_NO_DEFAULT IS set AND STARTUP_ENTRYPOINT is required**

```rust
else if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
    startup.push("// run startup");
    startup.push("var __webpack_exports__ = __webpack_require__.X();");
}
```

**Generated Code Example**:
```javascript
var __webpack_exports__ = __webpack_require__.X();
```

**The .X() function** (from startup_entrypoint.ejs or startup_entrypoint_with_async.ejs):

**Synchronous version** (startup_entrypoint.ejs):
```javascript
__webpack_require__.X = function(result, chunkIds, fn) {
    var moduleId = chunkIds;
    if (!fn) chunkIds = result, fn = function() { 
        return __webpack_require__(__webpack_require__.s = moduleId); 
    }
    chunkIds.map(__webpack_require__.e, __webpack_require__)
    var r = fn();
    return r === undefined ? result : r;
}
```

**Async version** (startup_entrypoint_with_async.ejs):
```javascript
__webpack_require__.X = function(result, chunkIds, fn) {
    var moduleId = chunkIds;
    if (!fn) chunkIds = result, fn = function() { 
        return __webpack_require__(__webpack_require__.s = moduleId); 
    }
    return Promise.all(chunkIds.map(__webpack_require__.e, __webpack_require__)).then(function() {
        var r = fn();
        return r === undefined ? result : r;
    });
}
```

Key difference: Async version returns a Promise for chunk loading.

### Level 3c: Minimal STARTUP Path (Branch C - no entries)

**ONLY reached when STARTUP_NO_DEFAULT IS set, STARTUP_ENTRYPOINT is NOT set, but STARTUP IS required**

```rust
else if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
    header.push("__webpack_require__.x = function(){};");
    startup.push("var __webpack_exports__ = __webpack_require__.x();");
}
```

An empty stub that does nothing.

## Runtime Requirement Flow

### Where does STARTUP_ENTRYPOINT come from?

**array_push_callback_chunk_format.rs** (lines 56-60):
```rust
if compilation.options.experiments.mf_async_startup {
    runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
} else {
    runtime_requirements.insert(RuntimeGlobals::ON_CHUNKS_LOADED);
}
```

**Decision Point**: 
- **mf_async_startup=true** → STARTUP_ENTRYPOINT for async Module Federation
- **mf_async_startup=false** → ON_CHUNKS_LOADED for deferred/passive startup

### Where does STARTUP come from?

**startup_chunk_dependencies.rs** (line 38):
```rust
if compilation.chunk_graph
    .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey)
    && is_enabled_for_chunk {
    runtime_requirements.insert(RuntimeGlobals::STARTUP);
}
```

**Condition**: Entry modules have dependent chunks that need to be loaded.

## The generate_entry_startup Helper

**Location**: helpers.rs lines 181-273

This function generates the initial entry startup code BEFORE it's wrapped/called:

```rust
pub fn generate_entry_startup(
    compilation: &Compilation,
    chunk: &ChunkUkey,
    entries: &IdentifierLinkedMap<ChunkGroupUkey>,
    passive: bool,  // KEY: Controls STARTUP vs STARTUP_ENTRYPOINT
) -> BoxSource
```

**If passive=false (active, async MF)**:
```javascript
var __webpack_exports__ = __webpack_require__.X(0, [chunk_ids...], function() {
    return __webpack_exec__(module_id);
});
```

**If passive=true (passive, deferred)**:
```javascript
__webpack_require__.O(0, [chunk_ids...], function() {
    return __webpack_exec__(module_id);
});
var __webpack_exports__ = __webpack_require__.O();
```

**Parameter meaning**:
- `0` - unused result parameter
- `[chunk_ids...]` - chunks to load before executing
- `function()` - execution callback
- `.O()` called again to process deferred queue

## Generated Output Examples

### Example 1: Simple Entry (No Dependencies)

```javascript
// header
var __webpack_module_cache__ = {};
function __webpack_require__(moduleId) {
    // ... require logic ...
}

// startup (inline, no wrapping)
var __webpack_exports__ = __webpack_require__(0);
```

### Example 2: Entry with Dependent Chunks (Sync/STARTUP path)

```javascript
// header
var __webpack_module_cache__ = {};
function __webpack_require__(moduleId) {
    // ... require logic ...
}
__webpack_require__.x = function() {
    var __webpack_exports__ = {};
    // Load entry module and return exports
    __webpack_exports__ = __webpack_require__.O(
        __webpack_exports__,
        [1, 2],  // dependent chunk IDs
        function() { return __webpack_require__(0); }
    );
    return __webpack_exports__;
};

// startup
var __webpack_exports__ = __webpack_require__.x();
```

### Example 3: Module Federation Async (STARTUP_ENTRYPOINT path)

Configuration:
```javascript
experiments: { mfAsyncStartup: true }
```

Generated code:
```javascript
// header
__webpack_require__.X = function(result, chunkIds, fn) {
    var moduleId = chunkIds;
    if (!fn) chunkIds = result, fn = function() { 
        return __webpack_require__(__webpack_require__.s = moduleId); 
    }
    return Promise.all(chunkIds.map(__webpack_require__.e, __webpack_require__)).then(function() {
        var r = fn();
        return r === undefined ? result : r;
    });
}

// array-push-callback format wraps entry startup:
(globalObject["webpackChunk"] = globalObject["webpackChunk"] || []).push([[chunk_id],
    { /* modules */ },
    function(__webpack_require__) {
        var __webpack_exports__ = __webpack_require__.X(0, [1, 2], function() {
            return __webpack_require__(0);
        });
    }
]);
```

Returns a Promise!

## Critical Code Locations

| Component | File | Lines | Purpose |
|-----------|------|-------|---------|
| Main render_bootstrap | `rspack_plugin_javascript/src/plugin/mod.rs` | 227-552 | Decision tree entry point |
| Entry startup generation | `rspack_plugin_runtime/src/helpers.rs` | 181-273 | Generates startup code with passive flag |
| Array-push format | `rspack_plugin_runtime/src/array_push_callback_chunk_format.rs` | 56-60 | Sets STARTUP_ENTRYPOINT vs ON_CHUNKS_LOADED |
| Startup dependencies | `rspack_plugin_runtime/src/startup_chunk_dependencies.rs` | 25-47 | Sets STARTUP when chunks have dependencies |
| STARTUP template | `rspack_plugin_runtime/src/runtime_module/runtime/startup_chunk_dependencies.ejs` | - | Wraps next startup |
| ON_CHUNKS_LOADED template | `rspack_plugin_runtime/src/runtime_module/runtime/on_chunk_loaded.ejs` | - | Deferred chunk callback handler |
| STARTUP_ENTRYPOINT (sync) | `rspack_plugin_runtime/src/runtime_module/runtime/startup_entrypoint.ejs` | - | Sync MF entry point |
| STARTUP_ENTRYPOINT (async) | `rspack_plugin_runtime/src/runtime_module/runtime/startup_entrypoint_with_async.ejs` | - | Async MF entry point with Promises |
| Runtime globals | `rspack_core/src/runtime_globals.rs` | 185, 201, 328, 320 | Definitions of STARTUP (1<<42), STARTUP_ENTRYPOINT (1<<34), name mappings |

## Decision Flowchart (ASCII)

```
START: render_bootstrap called
  │
  ├─ STARTUP_NO_DEFAULT set?
  │  ├─ NO (normal case)
  │  │  └─ chunk.has_entry_module()?
  │  │     ├─ YES
  │  │     │  └─ Build buf2 with module execution logic
  │  │     │     ├─ Check for bailouts (disable inline)
  │  │     │     ├─ Case: Entry has dependent chunks
  │  │     │     │  └─ Use __webpack_require__.O(chunks, fn)
  │  │     │     ├─ Case: use_require flag
  │  │     │     │  └─ Use __webpack_require__(moduleId)
  │  │     │     └─ Case: Direct execution
  │  │     │        └─ __webpack_modules__[moduleId](...)
  │  │     ├─ ON_CHUNKS_LOADED required?
  │  │     │  └─ Update __webpack_exports__ = __webpack_require__.O(...)
  │  │     └─ STARTUP required?
  │  │        ├─ YES: Wrap buf2 in __webpack_require__.x = function() {...}
  │  │        │        Call: var __webpack_exports__ = __webpack_require__.x();
  │  │        └─ NO: Inline buf2 directly into startup
  │  │     
  │  └─ NO (entry modules exist)
  │     └─ STARTUP required?
  │        └─ Define empty: __webpack_require__.x = function(){};
  │
  ├─ YES (STARTUP_NO_DEFAULT set)
  │  ├─ STARTUP_ENTRYPOINT required?
  │  │  └─ YES: Call var __webpack_exports__ = __webpack_require__.X();
  │  │         (determined by mf_async_startup experiment)
  │  │         Returns Promise if async version
  │  │
  │  └─ NO: STARTUP required?
  │     └─ YES: Define empty __webpack_require__.x = function(){};
  │             Call: var __webpack_exports__ = __webpack_require__.x();
  │
  └─ RETURN: RenderBootstrapResult {
     header: [...],    // __webpack_require__ definition, STARTUP function def
     startup: [...],   // var __webpack_exports__ = ...() call
     allow_inline_startup: bool
  }
```

## Key Insights

### 1. Mutual Exclusivity

The three startup methods are **mutually exclusive** at the render stage:
- In Branch A1a: STARTUP wrapping is used
- In Branch A1b: Direct inline (if STARTUP not required)
- In Branch B: STARTUP_ENTRYPOINT is used
- In Branch C: Empty STARTUP is defined

Only ONE `.x()` or `.X()` or inline code is generated per chunk.

### 2. The Inline Optimization

When `allow_inline_startup=true` and no STARTUP function is required, the code is **inlined directly** instead of wrapped:

```javascript
// Inlined (no wrapper)
var __webpack_exports__ = __webpack_require__(0);

// vs Wrapped (with STARTUP)
__webpack_require__.x = function() { 
    return __webpack_exports__ = __webpack_require__(0); 
};
var __webpack_exports__ = __webpack_require__.x();
```

The wrapper adds overhead but allows reuse of startup logic.

### 3. ON_CHUNKS_LOADED Double Call

The ON_CHUNKS_LOADED pattern has two calls:

```javascript
__webpack_require__.O(0, [chunks], function() { return __webpack_require__(0); });
var __webpack_exports__ = __webpack_require__.O();  // Process queue
```

First call **registers** the callback, second call **executes** deferred handlers.

### 4. Module Federation Async Mode

When `mf_async_startup=true`:
- STARTUP_ENTRYPOINT replaces ON_CHUNKS_LOADED
- Uses Promise-based async waiting (if async template used)
- Returns Promise instead of direct value
- Allows concurrent chunk loading

### 5. The "passive" Flag

In `generate_entry_startup()`:
- `passive=false` → Uses STARTUP_ENTRYPOINT (active, async)
- `passive=true` → Uses ON_CHUNKS_LOADED (passive, deferred)

This is set by array_push_callback_chunk_format.rs based on `mf_async_startup` config.

## Conclusion

The rspack startup bootstrap system is a **layered conditional system** that determines execution strategy based on:

1. **Configuration**: `experiments.mfAsyncStartup`
2. **Module Graph**: Entry dependencies and chunks
3. **Runtime Requirements**: What globals and features are needed
4. **Optimization Constraints**: Can code be inlined?

The three final output paths are:

| Path | Global | Behavior | Use Case |
|------|--------|----------|----------|
| STARTUP | `__webpack_require__.x()` | Synchronous function wrapper | When entry has dependent chunks to load |
| STARTUP_ENTRYPOINT | `__webpack_require__.X()` | Promise-based (async) or sync | Module Federation async startup |
| Inline | Direct code | Execute without wrapper | Simple entries without dependencies |

Each path optimizes for different scenarios: function reusability, async concurrency, or code size minimization.
