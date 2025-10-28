# Module Federation Runtime - Quick Reference & Implementation Checklist

**Date**: October 27, 2025

## Quick Answers to Key Questions

### Q1: Does federation have a custom "federation-entry-startup" requirement?
**A**: NO. Federation uses existing STARTUP (sync) and STARTUP_ENTRYPOINT (async) requirements.

### Q2: Where are remotes/consumes chunk handlers registered?
**A**: In runtime module generation:
- **Remotes**: `RemoteRuntimeModule::generate()` creates `__webpack_require__.f.remotes` handler
- **Consumes**: `ConsumeSharedRuntimeModule::generate()` creates `__webpack_require__.f.consumes` handler
- Both are called during `__webpack_require__.e(chunkId)` (ensure chunk)

### Q3: Can we add Promise.all wrapping for federation startup?
**A**: YES. Two approaches:
1. **Option A (Recommended)**: Extend EmbedFederationRuntimeModule to wrap in Promise.all when mf_async_startup=true
2. **Option B**: Create new FEDERATION_STARTUP requirement (more overhead)

### Q4: How does EmbedFederationRuntimeModule work?
**A**: It creates a "prevStartup wrapper" that:
1. Saves original `__webpack_require__.x` (or `.X`)
2. Replaces it with a function that executes federation modules first
3. Then calls the original startup
4. Runs only once (hasRun flag)

---

## Implementation Checklist for Promise.all Wrapping

### Phase 1: Detection & Analysis (2 files)

- [ ] **File**: `embed_federation_runtime_module.rs`
  - [ ] Analyze collected federation dependency IDs
  - [ ] Check if any depend on chunks (ENSURE_CHUNK requirement)
  - [ ] Determine if async wrapping needed
  - [ ] Set flag: `should_wrap_in_promise_all`

- [ ] **File**: `module_federation_runtime_plugin.rs`
  - [ ] Early detection of federation deps needing async handling
  - [ ] Add ENSURE_CHUNK_INCLUDE_ENTRIES when federation deps present
  - [ ] Signal EmbedFederationRuntimeModule about async needs

### Phase 2: Code Generation (1 file)

- [ ] **File**: `embed_federation_runtime_module.rs::generate()`
  - [ ] If `should_wrap_in_promise_all && mf_async_startup`:
    - [ ] Create Promise.all that ensures federation chunk dependencies
    - [ ] Execute federation modules inside `.then()`
    - [ ] Return promise chain
  - [ ] If sync or no dependencies:
    - [ ] Use existing synchronous wrapper
    - [ ] No promise wrapping needed

### Phase 3: Testing (Consider)

- [ ] Test with mf_async_startup=true, federation deps with chunks
- [ ] Test with mf_async_startup=false (should not wrap)
- [ ] Test with federation deps without chunk dependencies (no wrapping)
- [ ] Test error handling in promise chain

---

## Key Files & Line References

### Critical Runtime Module Files

| File | Purpose | Key Function | Lines |
|------|---------|--------------|-------|
| `embed_federation_runtime_module.rs` | **Federation startup wrapper** | `generate()` | 45-113 |
| `module_federation_runtime_plugin.rs` | **Federation requirements** | `additional_tree_runtime_requirements()` | 39-67 |
| `remote_runtime_module.rs` | Remotes handler registration | `generate()` | 40-108 |
| `consume_shared_runtime_module.rs` | Consumes handler registration | `generate()` | 39-155 |
| `share_runtime_module.rs` | Sharing initialization | `generate()` | 32-122 |
| `federation_data_runtime_module.rs` | Federation data | `federation_runtime_template()` | 51-105 |
| `embed_federation_runtime_plugin.rs` | Federation plugin orchestration | `render_startup()` | 163-225 |

### Generated JavaScript Files

| File | Contains |
|------|----------|
| `remotesLoading.js` | Remotes handler implementation |
| `consumesLoading.js` | Consumes handler implementation |
| `consumesCommon.js` | Shared consumes utilities |
| `consumesInitial.js` | Initial consumes setup |
| `initializeSharing.js` | Sharing initialization logic |

---

## Current (No Promise.all) Flow

```javascript
// Current wrapper (synchronous)
var prevStartup = __webpack_require__.x;  // or .X
var hasRun = false;
__webpack_require__.x = function() {
  if (!hasRun) {
    hasRun = true;
    // Federation modules execute synchronously
    __webpack_require__(123);  // Fed runtime
    __webpack_require__(124);  // Fed runtime
  }
  return typeof prevStartup === 'function' ? prevStartup() : undefined;
};
```

### Limitation
- Federation modules execute **synchronously**
- Even though STARTUP_ENTRYPOINT CAN be async, we don't use that capability
- Blocks on federation module loading instead of loading in parallel

---

## Proposed (With Promise.all) Flow

```javascript
// Proposed wrapper (async with Promise.all)
var prevStartup = __webpack_require__.X;
var hasRun = false;
var fedPromise;
__webpack_require__.X = function() {
  if (!hasRun) {
    hasRun = true;
    // Load chunks in parallel, then federation modules
    fedPromise = Promise.all([
      __webpack_require__.e(1),  // Remote chunk
      __webpack_require__.e(2)   // Share chunk
    ]).then(function() {
      // Federation modules execute after chunks loaded
      __webpack_require__(123);  // Fed runtime
      __webpack_require__(124);  // Fed runtime
      
      // Call original startup
      return typeof prevStartup === 'function' ? prevStartup() : undefined;
    });
    return fedPromise;
  }
  return fedPromise || (typeof prevStartup === 'function' ? prevStartup() : undefined);
};
```

### Benefits
- Federation chunks load in parallel with application startup
- Promise.all coordinates multiple chunk loads
- Non-blocking for other module loading
- True async Module Federation startup

---

## Handler Registration Details

### Remotes Handler (`__webpack_require__.f.remotes`)

**Registered in**: `RemoteRuntimeModule::generate()`

**Triggered when**: `__webpack_require__.e(chunkId)` is called and chunk has remotes

**Process**:
```
1. Check if chunkId has remotes in chunkMapping
2. For each remote module ID:
   a. Load external module: __webpack_require__(externalModuleId)
   b. Initialize sharing scope
   c. Get remote container
   d. Extract module factory from container
   e. Install factory in module cache
3. Push promises to array for async coordination
```

**Promise Chain**:
```
handleFunction → onExternal → onInitialized → onFactory
                    ↓            ↓              ↓
                Load ext    Get remote    Install factory
```

### Consumes Handler (`__webpack_require__.f.consumes`)

**Registered in**: `ConsumeSharedRuntimeModule::generate()`

**Triggered when**: `__webpack_require__.e(chunkId)` is called and chunk has consumes

**Process**:
```
1. Check if chunkId has consumes in chunkMapping
2. For each consumed module ID:
   a. Check if already loaded
   b. Call resolveHandler with consume data
   c. If returns promise: chain with .then(onFactory)['catch'](onError)
   d. If sync: call onFactory immediately
   e. Install module factory
3. Push promises to array for async coordination
```

### How Both Work Together

In `__webpack_require__.e()`:
```javascript
var promises = [];

// Call all handlers
__webpack_require__.f.remotes(chunkId, promises);
__webpack_require__.f.consumes(chunkId, promises);
__webpack_require__.f.modules(chunkId, promises);

// Wait for all to complete
return Promise.all(promises).then(function() { /* chunk loaded */ });
```

---

## Critical Interactions

### 1. EmbedFederationRuntimeModule ↔ Startup

- **EmbedFederationRuntimeModule** (Stage 11) wraps either:
  - `__webpack_require__.x` (sync startup)
  - `__webpack_require__.X` (async startup)
  
- **Decides**: Lines 92-96
  ```rust
  let startup = if compilation.options.experiments.mf_async_startup {
    RuntimeGlobals::STARTUP_ENTRYPOINT.name()  // ".X"
  } else {
    RuntimeGlobals::STARTUP.name()  // ".x"
  };
  ```

- **Impact**: Can return promise if `.X` used, but currently doesn't

### 2. FederationDataRuntimeModule ↔ Other Runtime Modules

- **Always added** by ModuleFederationRuntimePlugin
- **Provides**: `__webpack_require__.federation.chunkMatcher`, `rootOutputDir`
- **Used by**: RemoteRuntimeModule, ConsumeSharedRuntimeModule
- **Stage**: Normal (early)

### 3. RemoteRuntimeModule ↔ ConsumeSharedRuntimeModule

- **Both** register handlers: `__webpack_require__.f.*`
- **Both** require: MODULE, SHARE_SCOPE_MAP, INITIALIZE_SHARING
- **Execution order**: Both called in ensureChunk, order via stage
  - RemoteRuntimeModule (Stage 10)
  - ConsumeSharedRuntimeModule (Stage Attach)
  - ShareRuntimeModule (Stage Normal)

### 4. render_startup Hook ↔ EmbedFederationRuntimeModule

- **render_startup** (in EmbedFederationRuntimePlugin) appends startup call
- **Only for**: Entry chunks WITHOUT runtime (delegate to runtime)
- **Appends**: `__webpack_require__.x();` or `__webpack_require__.X();`
- **EmbedFederationRuntimeModule**: Wraps that function

---

## Runtime Requirements Summary

### Added by ModuleFederationRuntimePlugin
```rust
- FederationDataRuntimeModule (always)
- STARTUP_ENTRYPOINT (if mf_async_startup && has_runtime && has_entry_modules)
- ENSURE_CHUNK (if above)
- ENSURE_CHUNK_INCLUDE_ENTRIES (if above)
```

### Added by EmbedFederationRuntimePlugin
```rust
- STARTUP_ENTRYPOINT (if mf_async_startup && (has_runtime || has_entry_modules))
- STARTUP (if !mf_async_startup && (has_runtime || has_entry_modules))
- EmbedFederationRuntimeModule (if has_runtime && has_federation_deps)
```

### Added by ConsumeSharedPlugin
```rust
- MODULE, MODULE_CACHE, MODULE_FACTORIES_ADD_ONLY
- SHARE_SCOPE_MAP, INITIALIZE_SHARING, HAS_OWN_PROPERTY
- ConsumeSharedRuntimeModule
```

### Added by ContainerReferencePlugin
```rust
- MODULE, MODULE_FACTORIES_ADD_ONLY, HAS_OWN_PROPERTY
- INITIALIZE_SHARING, SHARE_SCOPE_MAP
- RemoteRuntimeModule (if ENSURE_CHUNK_HANDLERS required)
```

---

## Next Steps for Implementation

### Immediate (Understanding)
1. Read full `MF_RUNTIME_ANALYSIS.md` for detailed architecture
2. Review `embed_federation_runtime_module.rs` - understand prevStartup pattern
3. Review `module_federation_runtime_plugin.rs` - understand requirement injection

### Short-term (Implementation)
1. Extend EmbedFederationRuntimeModule::generate() to detect async needs
2. Add Promise.all wrapping when mf_async_startup=true
3. Test with examples/basic (already has mfAsyncStartup: true)

### Testing Strategy
1. Verify federation deps load in parallel (check DevTools network tab)
2. Verify no race conditions (multiple startup calls)
3. Verify promise chain completes before entry module execution
4. Verify fallback to sync if mf_async_startup=false

