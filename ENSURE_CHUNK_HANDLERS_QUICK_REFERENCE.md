# ENSURE_CHUNK_HANDLERS Quick Reference

## Definition

**What:** `RuntimeGlobals::ENSURE_CHUNK_HANDLERS` is a bitflag constant that maps to `__webpack_require__.f`

**Where:** `/Users/zackjackson/rspack/crates/rspack_core/src/runtime_globals.rs`

**Maps To:** `__webpack_require__.f` - an object holding chunk loading handler functions

---

## Handler Registration Chain

```
EnsureChunkRuntimeModule initializes:
    __webpack_require__.f = {}

Then handlers are registered:
    __webpack_require__.f.consumes = function(chunkId, promises) { }
    __webpack_require__.f.remotes = function(chunkId, promises) { }
    __webpack_require__.f.css = function(chunkId, promises, fetchPriority) { }
    __webpack_require__.f.prefetch = function(chunkId, promises) { }
    __webpack_require__.f.preload = function(chunkId, promises) { }
    __webpack_require__.f.require = function(chunkId, promises) { }
    __webpack_require__.f.i = function(chunkId, promises) { }
    __webpack_require__.f.j = function(chunkId, promises) { }

Then invoked dynamically:
    __webpack_require__.e = function(chunkId, fetchPriority) {
        return Promise.all(
            Object.keys(__webpack_require__.f).reduce(function(promises, key) {
                __webpack_require__.f[key](chunkId, promises, fetchPriority);
                return promises;
            }, [])
        );
    }
```

---

## Handler Types and Files

| Handler | Type | File | Purpose |
|---------|------|------|---------|
| **consumes** | Module Federation | `consumesLoading.js` | Load consumed shared dependencies |
| **remotes** | Module Federation | `remotesLoading.js` | Load remote modules |
| **css** | CSS Loading | `css_loading_with_loading.ejs` | Load CSS chunks |
| **prefetch** | Prefetch/Preload | `chunk_prefetch_trigger.ejs` | Prefetch child chunks |
| **preload** | Prefetch/Preload | `chunk_preload_trigger.ejs` | Preload child chunks |
| **require** | Node.js Loading | `require_chunk_loading_with_loading.ejs` | Require() chunk loading |
| **i** | Web Worker | `import_scripts_chunk_loading.ejs` | importScripts loading |
| **j** | Browser Loading | `jsonp_chunk_loading.ejs` | JSONP chunk loading |

---

## Key Files

### Core Definition
- `/Users/zackjackson/rspack/crates/rspack_core/src/runtime_globals.rs` (Lines 39-42, 292)

### Runtime Plugin (Insertion Logic)
- `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_plugin.rs` (Lines 308-319)

### Module Federation
- `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs`
- `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/sharing/consumesLoading.js`
- `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs`
- `/Users/zackjackson/rspack/crates/rspack_plugin_mf/src/container/remotesLoading.js`

### Core Chunk Loading
- `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_module/ensure_chunk.rs`
- `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/runtime_module/runtime/ensure_chunk.ejs`
- `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/jsonp_chunk_loading.rs`

### CSS Loading
- `/Users/zackjackson/rspack/crates/rspack_plugin_css/src/runtime/css_loading_with_loading.ejs`

---

## Access Pattern for Startup Code

### Initialize
```javascript
__webpack_require__.f = {};
```

### Register
```javascript
__webpack_require__.f.consumes = function(chunkId, promises) {
  // implementation
};
```

### Invoke
```javascript
__webpack_require__.e(chunkId, fetchPriority).then(function() {
  // chunk is loaded via all handlers
});
```

### Inside __webpack_require__.e
```javascript
Object.keys(__webpack_require__.f).reduce(function(promises, key) {
  __webpack_require__.f[key](chunkId, promises, fetchPriority);
  return promises;
}, [])
```

---

## When Is ENSURE_CHUNK_HANDLERS Required?

1. **Automatic:** When `ENSURE_CHUNK` is required AND chunk has async children
2. **Automatic:** When `ENSURE_CHUNK_INCLUDE_ENTRIES` is required
3. **Via Plugin:** When Module Federation (remotes/consumes) is configured
4. **Via Plugin:** When CSS chunk loading is enabled
5. **Via Plugin:** When chunk prefetch/preload is enabled

---

## Data Structures

### Consumes Data
```javascript
__webpack_require__.consumesLoadingData = {
  chunkMapping: {
    "1": ["10", "11"],        // chunkId -> [moduleIds]
  },
  moduleIdToConsumeDataMapping: {
    "10": {
      shareScope: "default",
      shareKey: "@org/utils",
      import: "@org/utils",
      requiredVersion: "^1.0.0",
      strictVersion: false,
      singleton: false,
      eager: false,
      fallback: "..." 
    }
  },
  initialConsumes: ["10"]
}
```

### Remotes Data
```javascript
__webpack_require__.remotesLoadingData = {
  chunkMapping: {
    "1": ["5", "6"],           // chunkId -> [moduleIds]
  },
  moduleIdToRemoteDataMapping: {
    "5": {
      shareScope: "default",
      name: "app1",
      externalModuleId: "3",
      remoteName: "@remote/app1"
    }
  }
}
```

---

## Module Federation Flow

### Consumes Handler
```
__webpack_require__.f.consumes(chunkId, promises)
  ↓
lookup chunkMapping[chunkId] -> [moduleIds]
  ↓
for each moduleId:
  - check if already installed
  - resolve shared dependency
  - call resolveHandler()
  - push promise to promises array
  ↓
return updated promises array
```

### Remotes Handler
```
__webpack_require__.f.remotes(chunkId, promises)
  ↓
lookup chunkMapping[chunkId] -> [moduleIds]
  ↓
for each moduleId:
  - track with __webpack_require__.R (current remote scope)
  - initialize sharing scope: __webpack_require__.I()
  - get remote module: external.get(name)
  - install module factory
  - push promise to promises array
  ↓
return updated promises array
```

---

## Critical Facts

1. **NOT async/promise by itself** - ENSURE_CHUNK_HANDLERS is just an object that holds handler functions

2. **Dynamic invocation** - `__webpack_require__.e()` calls handlers dynamically using `Object.keys()`

3. **Promise coordination** - Each handler may add promises to the array; all are awaited via `Promise.all()`

4. **Enhanced mode** - In "enhanced" builds, handlers can be stubbed with error-throwing functions if expected elsewhere

5. **Flexible** - New handlers can be added at runtime by assigning to `__webpack_require__.f.newType`

6. **Order independent** - Handlers are called in no particular order; `Object.keys()` determines iteration order

