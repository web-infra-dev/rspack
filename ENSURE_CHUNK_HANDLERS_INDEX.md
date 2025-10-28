# ENSURE_CHUNK_HANDLERS Research Index

Complete research on how `RuntimeGlobals::ENSURE_CHUNK_HANDLERS` is used throughout rspack.

## Documentation Files

1. **ENSURE_CHUNK_HANDLERS_QUICK_REFERENCE.md** - Start here for quick overview
   - Handler types and files
   - Access patterns
   - Data structures
   - When it's required

2. **ENSURE_CHUNK_HANDLERS_FINDINGS.md** - Comprehensive findings organized by question
   - Definition and location
   - Runtime modules that use it
   - Module Federation relationship
   - Handler pattern
   - Where it's inserted
   - Access methods in startup code

3. **ENSURE_CHUNK_HANDLERS_DETAILED_RESEARCH.md** - Deep technical analysis
   - Full file paths
   - Complete code snippets
   - Runtime requirements flow
   - Handler registration examples

---

## Key Answers

### 1. Where is ENSURE_CHUNK_HANDLERS defined?

**File:** `/Users/zackjackson/rspack/crates/rspack_core/src/runtime_globals.rs`

Lines 39-42 (definition):
```rust
const ENSURE_CHUNK_HANDLERS = 1 << 8;
```

Line 292 (JavaScript mapping):
```rust
R::ENSURE_CHUNK_HANDLERS => "__webpack_require__.f",
```

---

### 2. What runtime modules use it?

**Direct Usage:**
- EnsureChunkRuntimeModule - Initializes `__webpack_require__.f = {}`
- JsonpChunkLoadingRuntimeModule - Registers JSONP handler
- ModuleChunkLoadingRuntimeModule - Registers module loading handler
- RequireJsChunkLoadingRuntimeModule - Registers require() handler
- ImportScriptsChunkLoadingRuntimeModule - Registers importScripts handler
- ConsumeSharedRuntimeModule - Registers consumes handler
- RemoteRuntimeModule - Registers remotes handler
- CSS Loading modules - Register CSS handler
- Prefetch/Preload modules - Register prefetch/preload handlers

---

### 3. How does it relate to consumes and remotes in module federation?

**Both are registered as handlers under ENSURE_CHUNK_HANDLERS:**

Consumes handler:
```javascript
__webpack_require__.f.consumes = function(chunkId, promises) { ... }
```

Remotes handler:
```javascript
__webpack_require__.f.remotes = function(chunkId, promises) { ... }
```

Both follow the same pattern and are invoked dynamically when loading chunks.

**Files:**
- Consumes: `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consumesLoading.js`
- Remotes: `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remotesLoading.js`

---

### 4. Is there a pattern like __webpack_require__.f.consumes and __webpack_require__.f.remotes?

Yes\! All handlers follow this pattern:

```javascript
__webpack_require__.f.<handlerType> = function(chunkId, promises[, additionalArgs]) {
  // Handler implementation
  // May push promises or modify array
}
```

**All handler types:**
- f.consumes - Consumed shared dependencies
- f.remotes - Remote modules
- f.css - CSS chunks
- f.prefetch - Prefetch child chunks
- f.preload - Preload child chunks
- f.require - Node.js require() loading
- f.i - Web Worker importScripts
- f.j - Browser JSONP loading

---

### 5. Find all places where ENSURE_CHUNK_HANDLERS is inserted or used

**Insertion points:**

1. Runtime Plugin (automatic):
   - `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_plugin.rs` (Lines 308-319)
   - Inserted when ENSURE_CHUNK needed with async chunks
   - Inserted when ENSURE_CHUNK_INCLUDE_ENTRIES needed

2. Plugin-based insertion:
   - JsonpChunkLoadingPlugin: `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/jsonp_chunk_loading.rs`
   - ContainerReferencePlugin: `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/container_reference_plugin.rs`
   - CSS Plugin: `/Users/zackjackson/rspack/crates/rspack_plugin_css/src/plugin/impl_plugin_for_css_plugin.rs`
   - Extract CSS Plugin: `/Users/zackjackson/rspack/crates/rspack_plugin_extract_css/src/plugin.rs`

3. Runtime module checks:
   - EnsureChunkRuntimeModule: `ensure_chunk.rs` (Lines 55-73)
   - ConsumeSharedRuntimeModule: `consume_shared_runtime_module.rs` (Lines 138-152)
   - RemoteRuntimeModule: `remote_runtime_module.rs` (Lines 94-98)
   - All chunk loading modules check for ENSURE_CHUNK_HANDLERS

**Generated templates using ENSURE_CHUNK_HANDLERS:**
- ensure_chunk.ejs - Initializes and calls all handlers
- jsonp_chunk_loading.ejs - JSONP handler
- require_chunk_loading_with_loading.ejs - Require handler
- import_scripts_chunk_loading.ejs - ImportScripts handler
- chunk_prefetch_trigger.ejs - Prefetch handler
- chunk_preload_trigger.ejs - Preload handler
- css_loading_with_loading.ejs - CSS handler
- module_chunk_loading_with_loading.ejs - Module handler

---

### 6. How can we access these handlers in generated startup code?

**Access Methods:**

1. **Initialize:**
   ```javascript
   __webpack_require__.f = {};
   ```

2. **Register handler:**
   ```javascript
   __webpack_require__.f.consumes = function(chunkId, promises) { ... };
   ```

3. **Access directly:**
   ```javascript
   __webpack_require__.f.consumes
   __webpack_require__.f.remotes
   __webpack_require__.f.css
   ```

4. **Dynamic iteration:**
   ```javascript
   Object.keys(__webpack_require__.f).forEach(function(key) {
     __webpack_require__.f[key](chunkId, promises, fetchPriority);
   });
   ```

5. **In ensure_chunk function:**
   ```javascript
   __webpack_require__.e = function(chunkId, fetchPriority) {
     return Promise.all(
       Object.keys(__webpack_require__.f).reduce(function(promises, key) {
         __webpack_require__.f[key](chunkId, promises, fetchPriority);
         return promises;
       }, [])
     );
   };
   ```

6. **Conditional registration (enhanced mode):**
   ```javascript
   __webpack_require__.f.consumes = __webpack_require__.f.consumes || function() { 
     throw new Error("should have __webpack_require__.f.consumes") 
   };
   ```

---

## File Structure

```
rspack/
├── crates/
│   ├── rspack_core/
│   │   └── src/runtime_globals.rs                    # ENSURE_CHUNK_HANDLERS definition
│   ├── rspack_plugin_runtime/
│   │   ├── src/
│   │   │   ├── runtime_plugin.rs                     # Insertion logic
│   │   │   ├── jsonp_chunk_loading.rs                # JSONP plugin
│   │   │   └── runtime_module/
│   │   │       ├── ensure_chunk.rs                   # Main ensure_chunk module
│   │   │       ├── jsonp_chunk_loading.rs            # JSONP loading
│   │   │       ├── module_chunk_loading.rs           # Module loading
│   │   │       ├── require_js_chunk_loading.rs       # Require loading
│   │   │       ├── import_scripts_chunk_loading.rs   # ImportScripts loading
│   │   │       └── runtime/
│   │   │           ├── ensure_chunk.ejs              # Main template
│   │   │           ├── jsonp_chunk_loading.ejs       # JSONP template
│   │   │           ├── require_chunk_loading_with_loading.ejs
│   │   │           ├── import_scripts_chunk_loading.ejs
│   │   │           ├── chunk_prefetch_trigger.ejs
│   │   │           └── chunk_preload_trigger.ejs
│   ├── rspack_plugin_mf/
│   │   └── src/
│   │       ├── sharing/
│   │       │   ├── consume_shared_runtime_module.rs  # Consumes handler registration
│   │       │   └── consumesLoading.js                # Consumes implementation
│   │       └── container/
│   │           ├── container_reference_plugin.rs     # Remotes plugin
│   │           ├── remote_runtime_module.rs          # Remotes handler registration
│   │           └── remotesLoading.js                 # Remotes implementation
│   ├── rspack_plugin_css/
│   │   └── src/
│   │       ├── plugin/
│   │       │   └── impl_plugin_for_css_plugin.rs     # CSS plugin
│   │       └── runtime/
│   │           └── css_loading_with_loading.ejs      # CSS handler template
│   └── rspack_plugin_extract_css/
│       └── src/
│           ├── plugin.rs                             # Extract CSS plugin
│           └── runtime.rs                            # CSS runtime handler
```

---

## Understanding the Flow

```
1. COMPILATION PHASE
   ├─ RuntimePlugin analyzes requirements
   ├─ If ENSURE_CHUNK and has_async_chunks → insert ENSURE_CHUNK_HANDLERS
   └─ Plugins respond to ENSURE_CHUNK_HANDLERS
      ├─ JsonpChunkLoadingPlugin adds handlers
      ├─ ContainerReferencePlugin (for module federation)
      ├─ CSS Plugin adds CSS handler
      └─ Other plugins add their handlers

2. RUNTIME MODULE GENERATION
   ├─ EnsureChunkRuntimeModule generates:
   │  └─ __webpack_require__.f = {}
   │  └─ __webpack_require__.e = function() { ... }
   ├─ ConsumeSharedRuntimeModule generates:
   │  └─ __webpack_require__.f.consumes = function() { ... }
   ├─ RemoteRuntimeModule generates:
   │  └─ __webpack_require__.f.remotes = function() { ... }
   └─ Other modules generate their handlers

3. CODE GENERATION
   ├─ Templates are rendered with concrete handlers
   └─ __webpack_require__.f populated with all handlers

4. RUNTIME EXECUTION
   ├─ __webpack_require__.e(chunkId) called
   ├─ Iterates Object.keys(__webpack_require__.f)
   ├─ Calls each handler: f[key](chunkId, promises, fetchPriority)
   └─ Returns Promise.all() of collected promises
```

---

## Critical Code Examples

### Ensure Chunk Template (ensure_chunk.ejs)
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

### Consumes Handler (consumesLoading.js)
```javascript
__webpack_require__.f.consumes = function(chunkId, promises) {
  var moduleIdToConsumeDataMapping = __webpack_require__.consumesLoadingData.moduleIdToConsumeDataMapping
  var chunkMapping = __webpack_require__.consumesLoadingData.chunkMapping;
  if(__webpack_require__.o(chunkMapping, chunkId)) {
    chunkMapping[chunkId].forEach(function(id) {
      // ... resolution logic
      promises.push(...);
    });
  }
}
```

### Remotes Handler (remotesLoading.js)
```javascript
__webpack_require__.f.remotes = function (chunkId, promises) {
  var chunkMapping = __webpack_require__.remotesLoadingData.chunkMapping;
  var moduleIdToRemoteDataMapping = __webpack_require__.remotesLoadingData.moduleIdToRemoteDataMapping;
  if (__webpack_require__.o(chunkMapping, chunkId)) {
    chunkMapping[chunkId].forEach(function (id) {
      // ... remote loading logic
      promises.push(...);
    });
  }
};
```

---

## For Startup Code Implementation

To access ENSURE_CHUNK_HANDLERS in generated startup code:

1. Check if runtime needs handlers: `RuntimeGlobals::ENSURE_CHUNK_HANDLERS` in requirements
2. Access handlers via `__webpack_require__.f.consumes`, `__webpack_require__.f.remotes`, etc.
3. Iterate dynamically: `Object.keys(__webpack_require__.f)`
4. Call handler: `__webpack_require__.f[key](chunkId, promises, additionalArgs)`
5. Await all: `Promise.all(promises)`

The handlers are already registered in their respective runtime modules before startup code runs, so you can safely reference them.

