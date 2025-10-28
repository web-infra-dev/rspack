# ENSURE_CHUNK_HANDLERS Research Findings

## 1. Where is ENSURE_CHUNK_HANDLERS Defined?

**File:** `/Users/zackjackson/rspack/crates/rspack_core/src/runtime_globals.rs` (Lines 39-42, 292)

```rust
/// Definition (bitflag):
const ENSURE_CHUNK_HANDLERS = 1 << 8;

/// JavaScript name mapping:
R::ENSURE_CHUNK_HANDLERS => "__webpack_require__.f",
```

Maps to JavaScript global: `__webpack_require__.f` - an object holding chunk loading handlers.

---

## 2. What Runtime Modules Use It?

### Core Chunk Loading Modules:

1. **EnsureChunkRuntimeModule**
   - `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_module/ensure_chunk.rs` (Lines 55-73)
   - Initializes `__webpack_require__.f = {}` and creates `__webpack_require__.e()` function

2. **JsonpChunkLoadingRuntimeModule**
   - `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_module/jsonp_chunk_loading.rs` (Line 115)
   - Registers JSONP handler via `__webpack_require__.f.j`

3. **ModuleChunkLoadingRuntimeModule**, **RequireJsChunkLoadingRuntimeModule**, **ImportScriptsChunkLoadingRuntimeModule**
   - Check ENSURE_CHUNK_HANDLERS to determine handler registration

### Module Federation Modules:

4. **ConsumeSharedRuntimeModule**
   - `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs` (Lines 138-152)
   - Registers `__webpack_require__.f.consumes` handler

5. **RemoteRuntimeModule**
   - `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs` (Lines 94-98)
   - Registers `__webpack_require__.f.remotes` handler

### CSS & Prefetch/Preload:

6. **CSS Plugin**, **ChunkPrefetchPreloadFunctionRuntimeModule**
   - Register `__webpack_require__.f.css`, `__webpack_require__.f.prefetch`, `__webpack_require__.f.preload`

---

## 3. Relationship to Consumes and Remotes in Module Federation

### Consumes Pattern - `__webpack_require__.f.consumes`

**File:** `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consumesLoading.js`

```javascript
__webpack_require__.f.consumes = function(chunkId, promises) {
  var moduleIdToConsumeDataMapping = __webpack_require__.consumesLoadingData.moduleIdToConsumeDataMapping
  var chunkMapping = __webpack_require__.consumesLoadingData.chunkMapping;
  if(__webpack_require__.o(chunkMapping, chunkId)) {
    chunkMapping[chunkId].forEach(function(id) {
      // Resolve consumed shared dependencies
      if(__webpack_require__.o(installedModules, id)) return promises.push(installedModules[id]);
      var onFactory = function(factory) {
        installedModules[id] = 0;
        __webpack_require__.m[id] = function(module) {
          delete __webpack_require__.c[id];
          module.exports = factory();
        }
      };
      var onError = function(error) {
        delete installedModules[id];
        __webpack_require__.m[id] = function(module) {
          delete __webpack_require__.c[id];
          throw error;
        }
      };
      try {
        var promise = resolveHandler(moduleIdToConsumeDataMapping[id])();
        if(promise.then) {
          promises.push(installedModules[id] = promise.then(onFactory)['catch'](onError));
        } else onFactory(promise);
      } catch(e) { onError(e); }
    });
  }
}
```

**Data Structure:**
```javascript
__webpack_require__.consumesLoadingData = {
  chunkMapping: {},          // Maps chunkId -> [moduleIds]
  moduleIdToConsumeDataMapping: {},  // Maps moduleId -> {shareScope, shareKey, import, ...}
  initialConsumes: []        // Module IDs to consume initially
}
```

### Remotes Pattern - `__webpack_require__.f.remotes`

**File:** `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remotesLoading.js`

```javascript
__webpack_require__.f.remotes = function (chunkId, promises) {
  var chunkMapping = __webpack_require__.remotesLoadingData.chunkMapping;
  var moduleIdToRemoteDataMapping = __webpack_require__.remotesLoadingData.moduleIdToRemoteDataMapping;
  if (__webpack_require__.o(chunkMapping, chunkId)) {
    chunkMapping[chunkId].forEach(function (id) {
      var getScope = __webpack_require__.R;  // Current remote get scope
      if (!getScope) getScope = [];
      var data = moduleIdToRemoteDataMapping[id];
      if (getScope.indexOf(data) >= 0) return;
      getScope.push(data);
      if (data.p) return promises.push(data.p);
      // ... handle remote loading
      handleFunction(__webpack_require__, data.externalModuleId, 0, 0, onExternal, 1);
    });
  }
};
```

**Data Structure:**
```javascript
__webpack_require__.remotesLoadingData = {
  chunkMapping: {},          // Maps chunkId -> [moduleIds]
  moduleIdToRemoteDataMapping: {}  // Maps moduleId -> {shareScope, name, externalModuleId, remoteName}
}
```

---

## 4. Handler Pattern: `__webpack_require__.f.*`

All handlers follow this pattern and can be dynamically accessed:

| Property | Purpose | Signature |
|----------|---------|-----------|
| `f.consumes` | Load consumed shared modules | `function(chunkId, promises)` |
| `f.remotes` | Load remote modules | `function(chunkId, promises)` |
| `f.css` | Load CSS chunks | `function(chunkId, promises, fetchPriority)` |
| `f.prefetch` | Prefetch child chunks | `function(chunkId, promises)` |
| `f.preload` | Preload child chunks | `function(chunkId, promises)` |
| `f.require` | Require chunk loading | `function(chunkId, promises)` |
| `f.i` | importScripts chunk loading | `function(chunkId, promises)` |
| `f.j` | JSONP chunk loading | `function(chunkId, promises)` |

---

## 5. All Places Where ENSURE_CHUNK_HANDLERS is Inserted/Used

### Automatic Insertion (runtime_plugin.rs)

**File:** `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_plugin.rs` (Lines 308-319)

```rust
// When ENSURE_CHUNK is required and has async chunks
if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK) {
  let c = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let has_async_chunks = c.has_async_chunks(&compilation.chunk_group_by_ukey);
  if has_async_chunks {
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);  // <-- Inserted here
  }
  compilation.add_runtime_module(chunk_ukey, EnsureChunkRuntimeModule::default().boxed())?;
}

// When ENSURE_CHUNK_INCLUDE_ENTRIES is required
if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES) {
  runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);  // <-- Inserted here
}
```

### Plugin-Based Insertion

**JsonpChunkLoadingPlugin** - `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/jsonp_chunk_loading.rs` (Lines 29-34)

```rust
RuntimeGlobals::ENSURE_CHUNK_HANDLERS if is_enabled_for_chunk => {
  has_jsonp_chunk_loading = true;
  runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
  runtime_requirements_mut.insert(RuntimeGlobals::LOAD_SCRIPT);
  runtime_requirements_mut.insert(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);
}
```

**ContainerReferencePlugin** - `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/container_reference_plugin.rs` (Lines 116-126)

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

**Conditional Registration in Modules:**

- ConsumeSharedRuntimeModule: Lines 138-152
- RemoteRuntimeModule: Lines 94-98
- CSS Plugin: `/Users/zackjackson/rspack/crates/rspack_plugin_css/src/runtime/mod.rs`
- Extract CSS Plugin: `/Users/zackjackson/rspack/crates/rspack_plugin_extract_css/src/runtime.rs`

---

## 6. How to Access Handlers in Generated Startup Code

### Template: Ensure Chunk (ensure_chunk.ejs)

**File:** `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_module/runtime/ensure_chunk.ejs`

```ejs
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

### Key Access Patterns:

**1. Direct handler access:**
```javascript
__webpack_require__.f.consumes
__webpack_require__.f.remotes
__webpack_require__.f.css
```

**2. Dynamic handler invocation:**
```javascript
Object.keys(__webpack_require__.f).reduce(function(promises, key) {
  __webpack_require__.f[key](chunkId, promises, fetchPriority);
  return promises;
}, [])
```

**3. Conditional handler registration (enhanced mode):**

From ConsumeSharedRuntimeModule (Line 141):
```javascript
__webpack_require__.f.consumes = __webpack_require__.f.consumes || function() { 
  throw new Error("should have __webpack_require__.f.consumes") 
}
```

From RemoteRuntimeModule (Line 95):
```javascript
__webpack_require__.f.remotes = __webpack_require__.f.remotes || function() { 
  throw new Error("should have __webpack_require__.f.remotes"); 
}
```

### Handler Registration Examples:

**Require Handler** - `require_chunk_loading_with_loading.ejs`:
```javascript
__webpack_require__.f.require = function(chunkId, promises) {
  installedChunks[chunkId] = 1;
}
```

**Prefetch Handler** - `chunk_prefetch_trigger.ejs`:
```javascript
__webpack_require__.f.prefetch = function(chunkId, promises) {
  Promise.all(promises).then(function() {
    var chunks = chunkToChildrenMap[chunkId];
    Array.isArray(chunks) && chunks.map(prefetchChunk);
  });
};
```

**ImportScripts Handler** - `import_scripts_chunk_loading.ejs`:
```javascript
__webpack_require__.f.i = function(chunkId, promises) {
  // importScripts implementation
};
```

**CSS Handler** - `css_loading_with_loading.ejs`:
```javascript
__webpack_require__.f.css = function(chunkId, promises, fetchPriority) {
  // CSS chunk loading implementation
};
```

---

## Summary: Access Pattern for Startup Code

**In generated startup code, to access ENSURE_CHUNK_HANDLERS:**

```javascript
// 1. Initialize if needed:
__webpack_require__.f = {};

// 2. Register handlers as generated by modules:
__webpack_require__.f.consumes = function(chunkId, promises) { /* ... */ };
__webpack_require__.f.remotes = function(chunkId, promises) { /* ... */ };
__webpack_require__.f.css = function(chunkId, promises, fetchPriority) { /* ... */ };

// 3. Invoke in ensure_chunk function:
__webpack_require__.e = function(chunkId, fetchPriority) {
  return Promise.all(
    Object.keys(__webpack_require__.f).reduce(function(promises, key) {
      __webpack_require__.f[key](chunkId, promises, fetchPriority);
      return promises;
    }, [])
  );
};

// 4. Call during chunk loading:
__webpack_require__.e(1).then(function() {
  // Chunk 1 is loaded via all registered handlers
});
```

