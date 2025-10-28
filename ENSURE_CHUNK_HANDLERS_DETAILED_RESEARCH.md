# RuntimeGlobals::ENSURE_CHUNK_HANDLERS Research Summary

## 1. Definition and Location

### File: `/Users/zackjackson/rspack/crates/rspack_core/src/runtime_globals.rs`
**Lines 39-42:**
```rust
/**
 * an object with handlers to ensure a chunk
 */
const ENSURE_CHUNK_HANDLERS = 1 << 8;
```

**Line 292:**
```rust
R::ENSURE_CHUNK_HANDLERS => "__webpack_require__.f",
```

**Key Insight:** ENSURE_CHUNK_HANDLERS maps to the JavaScript global `__webpack_require__.f`, which is an object that holds handler functions for chunk loading.

---

## 2. Runtime Modules Using ENSURE_CHUNK_HANDLERS

### A. Core Chunk Loading Modules

1. **EnsureChunkRuntimeModule**
   - **File:** `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_module/ensure_chunk.rs`
   - **Lines 55-73:** Checks if ENSURE_CHUNK_HANDLERS is required
   - **Function:** Initializes `__webpack_require__.f` as an empty object and creates the chunk ensure function
   
2. **JsonpChunkLoadingRuntimeModule**
   - **File:** `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_module/jsonp_chunk_loading.rs`
   - **Line 115:** Checks `runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)`
   - **Template:** References ENSURE_CHUNK_HANDLERS in jsonp_chunk_loading.ejs

3. **ModuleChunkLoadingRuntimeModule**
   - Checks for ENSURE_CHUNK_HANDLERS to determine if loading handler needed

4. **RequireJsChunkLoadingRuntimeModule**
   - Handles require() chunk loading with ENSURE_CHUNK_HANDLERS

5. **ImportScriptsChunkLoadingRuntimeModule**
   - **Template:** `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_module/runtime/import_scripts_chunk_loading.ejs`
   - **Line 14:** Registers `__webpack_require__.f.i` handler

### B. Module Federation Modules

1. **ConsumeSharedRuntimeModule**
   - **File:** `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs`
   - **Lines 138-143:** Registers `__webpack_require__.f.consumes` handler
   - **Lines 149-152:** Includes consumesLoading.js if ENSURE_CHUNK_HANDLERS is required
   
   Code snippet:
   ```rust
   if ChunkGraph::get_chunk_runtime_requirements(compilation, &chunk_ukey)
     .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
   {
     source += include_str!("./consumesLoading.js");
   }
   ```

2. **RemoteRuntimeModule**
   - **File:** `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs`
   - **Lines 94-98:** Registers `__webpack_require__.f.remotes` handler
   - **Code:**
   ```rust
   let remotes_loading_impl = if self.enhanced {
     "__webpack_require__.f.remotes = __webpack_require__.f.remotes || function() { throw new Error(\"should have __webpack_require__.f.remotes\"); }"
   } else {
     include_str!("./remotesLoading.js")
   };
   ```

### C. CSS Loading Module

1. **CSS Plugin**
   - **File:** `/Users/zackjackson/rspack/crates/rspack_plugin_css/src/runtime/mod.rs`
   - Registers `__webpack_require__.f.css` handler for CSS chunk loading

### D. Chunk Prefetch/Preload Modules

1. **ChunkPrefetchPreloadFunctionRuntimeModule**
   - Registers `__webpack_require__.f.prefetch` and `__webpack_require__.f.preload` handlers

---

## 3. Relationship to Consumes and Remotes in Module Federation

### Consumes Pattern (`__webpack_require__.f.consumes`)

**File:** `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consumesLoading.js`

Handler signature:
```javascript
__webpack_require__.f.consumes = function(chunkId, promises) {
  // Implementation handles loading of consumed shared modules
}
```

**Key Operations:**
- Maps chunk IDs to module IDs needing consumed dependencies
- Resolves shared module dependencies
- Handles module installation and error cases
- Pushes promises to ensure chunks are loaded

**Data Structure:**
```javascript
__webpack_require__.consumesLoadingData = {
  chunkMapping: {},          // Maps chunk IDs to module IDs
  moduleIdToConsumeDataMapping: {},  // Maps module IDs to consume metadata
  initialConsumes: []        // Module IDs to consume initially
}
```

### Remotes Pattern (`__webpack_require__.f.remotes`)

**File:** `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remotesLoading.js`

Handler signature:
```javascript
__webpack_require__.f.remotes = function(chunkId, promises) {
  // Implementation handles loading of remote modules
}
```

**Key Operations:**
- Maps chunk IDs to module IDs needing remotes
- Uses `__webpack_require__.R` (current remote get scope) to track loaded remotes
- Handles fallback mechanisms and error cases
- Initializes sharing scope and gets remote exports

**Data Structure:**
```javascript
__webpack_require__.remotesLoadingData = {
  chunkMapping: {},          // Maps chunk IDs to remote module IDs
  moduleIdToRemoteDataMapping: {}  // Maps module IDs to remote metadata
}
```

---

## 4. Handler Registration Pattern: `__webpack_require__.f.*`

All handlers follow a consistent registration pattern using property assignment on the handlers object.

### Handler Properties

| Property | Module | Purpose |
|----------|--------|---------|
| `f.consumes` | ConsumeSharedRuntimeModule | Load consumed shared modules |
| `f.remotes` | RemoteRuntimeModule | Load remote modules |
| `f.css` | CSS Plugin | Load CSS chunks |
| `f.prefetch` | ChunkPrefetchPreloadFunctionRuntimeModule | Prefetch child chunks |
| `f.preload` | ChunkPrefetchPreloadFunctionRuntimeModule | Preload child chunks |
| `f.require` | RequireChunkLoadingRuntimeModule | Require chunk loading |
| `f.i` | ImportScriptsChunkLoadingRuntimeModule | importScripts chunk loading |
| `f.j` | JsonpChunkLoadingRuntimeModule | JSONP chunk loading |

### Handler Function Signature

All handlers follow this pattern:
```javascript
__webpack_require__.f.<type> = function(chunkId, promises[, additionalArgs]) {
  // Handler implementation
  // Typically pushes promises or modifies promises array
}
```

**Parameters:**
- `chunkId`: The ID of the chunk being loaded
- `promises`: Array to accumulate loading promises
- `additionalArgs`: Optional arguments (e.g., `fetchPriority` for CSS)

---

## 5. Where ENSURE_CHUNK_HANDLERS is Inserted/Used

### A. Automatic Insertion Points

**File:** `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_plugin.rs`

**Lines 308-315:**
```rust
if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK) {
  let c = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let has_async_chunks = c.has_async_chunks(&compilation.chunk_group_by_ukey);
  if has_async_chunks {
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
  }
  compilation.add_runtime_module(chunk_ukey, EnsureChunkRuntimeModule::default().boxed())?;
}
```

**Lines 317-319:**
```rust
if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES) {
  runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
}
```

### B. Conditional Insertion by Plugins

**JsonpChunkLoadingPlugin** - `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/jsonp_chunk_loading.rs`
```rust
RuntimeGlobals::ENSURE_CHUNK_HANDLERS if is_enabled_for_chunk => {
  has_jsonp_chunk_loading = true;
  runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
  runtime_requirements_mut.insert(RuntimeGlobals::LOAD_SCRIPT);
  runtime_requirements_mut.insert(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);
}
```

**ContainerReferencePlugin** - `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/container_reference_plugin.rs`
```rust
if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) {
  runtime_requirements_mut.insert(RuntimeGlobals::MODULE);
  runtime_requirements_mut.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
  runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
  runtime_requirements_mut.insert(RuntimeGlobals::INITIALIZE_SHARING);
  runtime_requirements_mut.insert(RuntimeGlobals::SHARE_SCOPE_MAP);
  compilation.add_runtime_module(
    chunk_ukey,
    Box::new(RemoteRuntimeModule::new(self.options.enhanced)),
  )?;
}
```

---

## 6. Generated Template Examples

### A. Ensure Chunk Template
**File:** `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_module/runtime/ensure_chunk.ejs`

```javascript
<%- ENSURE_CHUNK_HANDLERS %> = {};
// This file contains only the entry chunk.
// The chunk loading function for additional chunks
<%- ENSURE_CHUNK %> = <%- basicFunction(_args) %> {
  return Promise.all(
    Object.keys(<%- ENSURE_CHUNK_HANDLERS %>).reduce(<%- basicFunction("promises, key") %> {
      <%- ENSURE_CHUNK_HANDLERS %>[key](chunkId, promises<%- _fetch_priority %>);
      return promises;
    }, [])
  );
};
```

**Generated JavaScript:**
```javascript
__webpack_require__.f = {};
__webpack_require__.e = function(chunkId, fetchPriority) {
  return Promise.all(
    Object.keys(__webpack_require__.f).reduce(function(promises, key) {
      __webpack_require__.f[key](chunkId, promises, fetchPriority);
      return promises;
    }, [])
  );
};
```

### B. Handler Registration Examples

#### Consume Handler (consumesLoading.js)
```javascript
__webpack_require__.f.consumes = function(chunkId, promises) {
  var moduleIdToConsumeDataMapping = __webpack_require__.consumesLoadingData.moduleIdToConsumeDataMapping
  var chunkMapping = __webpack_require__.consumesLoadingData.chunkMapping;
  if(__webpack_require__.o(chunkMapping, chunkId)) {
    chunkMapping[chunkId].forEach(function(id) {
      // ... loading logic
    });
  }
}
```

#### Remote Handler (remotesLoading.js)
```javascript
__webpack_require__.f.remotes = function(chunkId, promises) {
  var chunkMapping = __webpack_require__.remotesLoadingData.chunkMapping;
  var moduleIdToRemoteDataMapping = __webpack_require__.remotesLoadingData.moduleIdToRemoteDataMapping;
  if(__webpack_require__.o(chunkMapping, chunkId)) {
    chunkMapping[chunkId].forEach(function(id) {
      // ... remote loading logic
    });
  }
};
```

#### CSS Handler (css_loading_with_loading.ejs)
```javascript
<%- ENSURE_CHUNK_HANDLERS %>.css = <%- basicFunction("chunkId, promises, fetchPriority") %> {
  // css chunk loading implementation
};
```

#### Prefetch Handler (chunk_prefetch_trigger.ejs)
```javascript
<%- ENSURE_CHUNK_HANDLERS %>.prefetch = <%- basicFunction("chunkId, promises") %> {
  Promise.all(promises).then(<%- basicFunction("") %> {
    var chunks = chunkToChildrenMap[chunkId];
    Array.isArray(chunks) && chunks.map(<%- PREFETCH_CHUNK %>);
  });
};
```

#### Require Handler (require_chunk_loading_with_loading.ejs)
```javascript
<%- ENSURE_CHUNK_HANDLERS %>.require = <%- basicFunction("chunkId, promises") %> {
  installedChunks[chunkId] = 1;
}
```

---

## 7. Accessing Handlers in Generated Startup Code

### A. Direct Handler Access Pattern

Within generated code, handlers can be accessed via:

```javascript
__webpack_require__.f.consumes
__webpack_require__.f.remotes
__webpack_require__.f.css
__webpack_require__.f.prefetch
__webpack_require__.f.require
```

### B. Dynamic Handler Invocation

The `ensure_chunk.ejs` template demonstrates dynamic invocation:

```javascript
Object.keys(__webpack_require__.f).reduce(function(promises, key) {
  __webpack_require__.f[key](chunkId, promises, fetchPriority);
  return promises;
}, [])
```

This pattern:
1. Iterates over all registered handlers using `Object.keys()`
2. Calls each handler dynamically with `[key]()` notation
3. Passes `chunkId` and `promises` array to each handler
4. Accumulates promises from all handlers

### C. Conditional Handler References

In "enhanced" mode (for faster startup), handlers may reference other modules:

From `consume_shared_runtime_module.rs` (lines 141):
```javascript
__webpack_require__.f.consumes = __webpack_require__.f.consumes || function() { 
  throw new Error("should have __webpack_require__.f.consumes") 
}
```

From `remote_runtime_module.rs` (lines 95):
```javascript
__webpack_require__.f.remotes = __webpack_require__.f.remotes || function() { 
  throw new Error("should have __webpack_require__.f.remotes"); 
}
```

This pattern allows:
- Detecting if a handler is already defined
- Providing fallback implementations
- Early error detection if expected handlers are missing

---

## 8. Key Files Summary

| File | Purpose |
|------|---------|
| `/Users/zackjackson/rspack/crates/rspack_core/src/runtime_globals.rs` | Defines ENSURE_CHUNK_HANDLERS constant |
| `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_plugin.rs` | Inserts ENSURE_CHUNK_HANDLERS when needed |
| `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_module/ensure_chunk.rs` | Initializes `__webpack_require__.f` object |
| `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_module/runtime/ensure_chunk.ejs` | Template for ensure_chunk implementation |
| `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs` | Registers consumes handler |
| `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consumesLoading.js` | Consumes handler implementation |
| `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs` | Registers remotes handler |
| `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remotesLoading.js` | Remotes handler implementation |
| `/Users/zackjackson/rspack/crates/rspack_plugin_css/src/runtime/css_loading_with_loading.ejs` | CSS handler template |

---

## 9. Runtime Requirements Flow

```
ENSURE_CHUNK (async chunks detected)
    ↓
ENSURE_CHUNK_HANDLERS (auto-inserted)
    ↓
Plugins respond to ENSURE_CHUNK_HANDLERS:
  - JsonpChunkLoadingPlugin: adds PUBLIC_PATH, LOAD_SCRIPT, etc.
  - ContainerReferencePlugin: adds MODULE, INITIALIZE_SHARING, etc.
  - CssPlugin: adds CSS loading handler
    ↓
Runtime modules registered:
  - EnsureChunkRuntimeModule: initializes __webpack_require__.f = {}
  - JsonpChunkLoadingRuntimeModule: registers f.j handler
  - ConsumeSharedRuntimeModule: registers f.consumes handler
  - RemoteRuntimeModule: registers f.remotes handler
  - CssLoadingRuntimeModule: registers f.css handler
    ↓
Generated output:
  - __webpack_require__.f = {};
  - __webpack_require__.f.consumes = function() { ... }
  - __webpack_require__.f.remotes = function() { ... }
  - __webpack_require__.f.css = function() { ... }
  - __webpack_require__.e = function(chunkId) { 
      return Promise.all(Object.keys(__webpack_require__.f).reduce(...))
    }
```

