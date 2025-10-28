# Module Federation Runtime Modules Architecture Analysis

**Date**: October 27, 2025  
**Branch**: feature/async-startup-runtime-promise  
**Crate**: rspack_plugin_mf  

## Executive Summary

This document provides a comprehensive analysis of how module federation runtime modules work in rspack, focusing on:

1. How federation adds custom runtime requirements
2. How consumes and remotes chunk handlers are registered
3. Federation's integration with async startup
4. Recommendations for Promise.all wrapping

### Key Finding
Module Federation uses a **hierarchical runtime module injection system** where:
- **EmbedFederationRuntimeModule** wraps startup execution
- **RemoteRuntimeModule** provides remotes loading logic
- **ConsumeSharedRuntimeModule** provides sharing/consumes resolution
- **ShareRuntimeModule** provides shared scope initialization
- **FederationDataRuntimeModule** provides base federation data

Federation does NOT currently use a custom "federation-entry-startup" requirement, but uses:
- **STARTUP_ENTRYPOINT** (when mf_async_startup enabled) 
- **STARTUP** (default synchronous wrapper)

---

## 1. Runtime Requirements Added by Federation

Federation plugins add runtime requirements through three mechanisms:

### A. Direct Runtime Requirement Insertion

**File**: `ModuleFederationRuntimePlugin::additional_tree_runtime_requirements()`  
**Location**: `/crates/rspack_plugin_mf/src/container/module_federation_runtime_plugin.rs:39-67`

```rust
#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ModuleFederationRuntimePlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  // Always add FederationDataRuntimeModule
  compilation.add_runtime_module(chunk_ukey, Box::<FederationDataRuntimeModule>::default())?;

  // When mf_async_startup is enabled, add STARTUP_ENTRYPOINT requirements
  if compilation.options.experiments.mf_async_startup {
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

    if chunk.has_runtime(&compilation.chunk_group_by_ukey)
      && compilation
        .chunk_graph
        .get_number_of_entry_modules(chunk_ukey)
        > 0
    {
      runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
    }
  }

  Ok(())
}
```

**Key Points**:
- FederationDataRuntimeModule is ALWAYS added to provide base federation data
- STARTUP_ENTRYPOINT is only added when `mf_async_startup` experiment is enabled
- This applies to runtime chunks with entry modules only

### B. EmbedFederationRuntimePlugin Requirements

**File**: `EmbedFederationRuntimePlugin::additional_chunk_runtime_requirements_tree()`  
**Location**: `/crates/rspack_plugin_mf/src/container/embed_federation_runtime_plugin.rs:56-90`

```rust
#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for EmbedFederationRuntimePlugin)]
async fn additional_chunk_runtime_requirements_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  // Check if chunk needs federation runtime support
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  let has_entry_modules = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0;

  // Federation is enabled for runtime chunks or entry chunks
  let is_enabled = has_runtime || has_entry_modules;

  if is_enabled {
    // Add STARTUP or STARTUP_ENTRYPOINT based on mf_async_startup experiment
    if compilation.options.experiments.mf_async_startup {
      runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
    } else {
      runtime_requirements.insert(RuntimeGlobals::STARTUP);
    }
  }

  Ok(())
}
```

**Key Points**:
- Adds either STARTUP_ENTRYPOINT (async) or STARTUP (sync) to federation-enabled chunks
- Triggered for both runtime chunks AND entry chunks with federation dependencies

### C. ConsumeSharedPlugin Requirements

**File**: `ConsumeSharedPlugin::additional_tree_runtime_requirements()`  
**Location**: `/crates/rspack_plugin_mf/src/sharing/consume_shared_plugin.rs:473-491`

```rust
#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ConsumeSharedPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  runtime_requirements.insert(RuntimeGlobals::MODULE);
  runtime_requirements.insert(RuntimeGlobals::MODULE_CACHE);
  runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
  runtime_requirements.insert(RuntimeGlobals::SHARE_SCOPE_MAP);
  runtime_requirements.insert(RuntimeGlobals::INITIALIZE_SHARING);
  runtime_requirements.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
  compilation.add_runtime_module(
    chunk_ukey,
    Box::new(ConsumeSharedRuntimeModule::new(self.options.enhanced)),
  )?;
  Ok(())
}
```

**Key Runtime Requirements Added**:
- `MODULE` - Module object
- `MODULE_CACHE` - Module caching
- `MODULE_FACTORIES_ADD_ONLY` - Factory functions
- `SHARE_SCOPE_MAP` - Scope mapping
- `INITIALIZE_SHARING` - Sharing initialization
- `HAS_OWN_PROPERTY` - Object property checking

### D. ContainerReferencePlugin (Remotes) Requirements

**File**: `ContainerReferencePlugin::runtime_requirements_in_tree()`  
**Location**: `/crates/rspack_plugin_mf/src/container/container_reference_plugin.rs:107-128`

```rust
#[plugin_hook(CompilationRuntimeRequirementInTree for ContainerReferencePlugin)]
async fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
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
  Ok(None)
}
```

**Key Points**:
- Triggered only when ENSURE_CHUNK_HANDLERS is required
- Adds MODULE and INITIALIZE_SHARING requirements for remote loading
- RemoteRuntimeModule registers `__webpack_require__.f.remotes` handler

---

## 2. How Consumes and Remotes Chunk Handlers Are Registered

Chunk handlers are registered through **runtime module generation** that creates handler functions.

### A. Remotes Handler Registration

**Generated Code**: `RemoteRuntimeModule::generate()`  
**Location**: `/crates/rspack_plugin_mf/src/container/remote_runtime_module.rs:40-108`

The RemoteRuntimeModule generates JavaScript that sets up the remotes loader:

```javascript
__webpack_require__.remotesLoadingData = {
  chunkMapping: { /* chunk -> module IDs */ },
  moduleIdToRemoteDataMapping: { /* module ID -> remote data */ }
};

__webpack_require__.f.remotes = function (chunkId, promises) {
  var chunkMapping = __webpack_require__.remotesLoadingData.chunkMapping;
  var moduleIdToRemoteDataMapping = __webpack_require__.remotesLoadingData.moduleIdToRemoteDataMapping;
  if (__webpack_require__.o(chunkMapping, chunkId)) {
    chunkMapping[chunkId].forEach(function (id) {
      var getScope = __webpack_require__.R;
      if (\!getScope) getScope = [];
      var data = moduleIdToRemoteDataMapping[id];
      // ... recursive promise handling for remote loading
      handleFunction(__webpack_require__, data.externalModuleId, 0, 0, onExternal, 1);
    });
  }
};
```

**Key Components**:
- `chunkMapping`: Maps chunks to their remote module IDs
- `moduleIdToRemoteDataMapping`: Maps module IDs to remote data (scope, name, external module)
- Handler function: Registered as `__webpack_require__.f.remotes` to handle chunk loading
- Promise-based chaining: Each remote loads dependencies sequentially

**Handler Chain**:
```
onExternal → onInitialized → onFactory
  ↓           ↓                ↓
Load ext → Get remote → Install factory
```

### B. Consumes Handler Registration

**Generated Code**: `ConsumeSharedRuntimeModule::generate()`  
**Location**: `/crates/rspack_plugin_mf/src/sharing/consume_shared_runtime_module.rs:39-155`

The ConsumeSharedRuntimeModule generates JavaScript that sets up the consumes resolver:

```javascript
__webpack_require__.consumesLoadingData = {
  chunkMapping: { /* chunk -> module IDs */ },
  moduleIdToConsumeDataMapping: { /* module ID -> consume data */ },
  initialConsumes: [] // For eager consumes
};

__webpack_require__.f.consumes = function(chunkId, promises) {
  var moduleIdToConsumeDataMapping = __webpack_require__.consumesLoadingData.moduleIdToConsumeDataMapping
  var chunkMapping = __webpack_require__.consumesLoadingData.chunkMapping;
  if(__webpack_require__.o(chunkMapping, chunkId)) {
    chunkMapping[chunkId].forEach(function(id) {
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

**Key Components**:
- `chunkMapping`: Maps chunks to consumed module IDs
- `moduleIdToConsumeDataMapping`: Maps module IDs to consume configuration
  - `shareScope`: Scope name
  - `shareKey`: Module key within scope
  - `import`: Import specifier
  - `requiredVersion`: Version constraint
  - `singleton`: Singleton flag
  - `eager`: Eager loading flag
  - `fallback`: Fallback module
- Handler function: Registered as `__webpack_require__.f.consumes`
- Promise support: Uses Promise.then for async resolution

**Resolution Flow**:
```
resolveHandler(consumeData)() → promise
  ↓
promise.then(onFactory)['catch'](onError)
  ↓
Install module factory with resolved export
```

### C. How Handlers Are Called

Chunk handlers are invoked through the **ensureChunk** mechanism:

When a chunk is requested:
1. Runtime calls `__webpack_require__.e(chunkId)` (ensure chunk)
2. This triggers all registered handlers: `__webpack_require__.f.*`
3. Handlers receive `(chunkId, promises)` array
4. Handlers can push promises to coordinate async work
5. When all promises resolve, chunk is considered loaded

**Handler Registration Points**:
- Remote handler: In RemoteRuntimeModule (stage 10)
- Consume handler: In ConsumeSharedRuntimeModule (stage Attach)
- Share handler: In ShareRuntimeModule (responds to SHARE_SCOPE_MAP requirement)

---

## 3. Federation Startup Integration with Async Mode

### Current Architecture (Without Promise.all)

**EmbedFederationRuntimeModule** creates a "prevStartup wrapper":

```javascript
var prevStartup = __webpack_require__.x;  // or .X if async
var hasRun = false;
__webpack_require__.x = function() {
  if (\!hasRun) {
    hasRun = true;
    // Execute federation runtime modules inline
    __webpack_require__(federation_module_id_1);
    __webpack_require__(federation_module_id_2);
  }
  if (typeof prevStartup === 'function') {
    return prevStartup();
  } else {
    console.warn('[MF] Invalid prevStartup');
  }
};
```

**File**: `/crates/rspack_plugin_mf/src/container/embed_federation_runtime_module.rs:90-111`

**Limitations**:
1. Federation modules execute **synchronously** even when `mf_async_startup = true`
2. No Promise wrapping for federation module dependencies
3. No concurrent loading of federation dependencies with entry modules
4. If federation modules return Promises, those are ignored

### Where the Decision Happens

**File**: `EmbedFederationRuntimeModule::generate()`  
**Lines**: 92-96

```rust
// Use STARTUP_ENTRYPOINT when mf_async_startup is enabled, otherwise use STARTUP
let startup = if compilation.options.experiments.mf_async_startup {
  RuntimeGlobals::STARTUP_ENTRYPOINT.name()
} else {
  RuntimeGlobals::STARTUP.name()
};
```

**Current Behavior**:
- When `mf_async_startup = true`: Wraps `__webpack_require__.X` (which CAN be async)
- But federation module execution is still **synchronous** - no Promise.all wrapping
- This means async benefits are lost at federation level

### Runtime Module Stages

Federation runtime modules execute in specific order via stages:

```
Stage 0-8: Basic runtime setup
Stage 9: (unused)
Stage 10: RemoteRuntimeModule (remotes loading)
Stage 11: EmbedFederationRuntimeModule (federation startup wrapping)
Stage Attach: ConsumeSharedRuntimeModule, ShareRuntimeModule
```

**Critical**: RemoteRuntimeModule executes BEFORE EmbedFederationRuntimeModule

---

## 4. All Runtime Modules in the MF Crate

### Complete Runtime Module Inventory

#### 1. **FederationDataRuntimeModule**
- **File**: `federation_data_runtime_module.rs`
- **Purpose**: Provides base federation data to runtime
- **Stage**: RuntimeModuleStage::Normal
- **Generated Output**: Federation configuration object
```javascript
if(\!__webpack_require__.federation){
    __webpack_require__.federation = {
        chunkMatcher: function(chunkId) { ... },
        rootOutputDir: "path/to/output"
    };
}
```
- **Always Added**: Yes (by ModuleFederationRuntimePlugin)

#### 2. **RemoteRuntimeModule**
- **File**: `remote_runtime_module.rs`
- **Purpose**: Handles remote module loading
- **Stage**: RuntimeModuleStage::Attach
- **Generated Output**: Remotes loading handler
```javascript
__webpack_require__.remotesLoadingData = { chunkMapping, moduleIdToRemoteDataMapping };
__webpack_require__.f.remotes = function(chunkId, promises) { ... };
```
- **Added When**: ENSURE_CHUNK_HANDLERS required (by ContainerReferencePlugin)
- **Enhanced Mode**: Can skip implementation if enhanced=true

#### 3. **ConsumeSharedRuntimeModule**
- **File**: `consume_shared_runtime_module.rs`
- **Purpose**: Handles shared module consumption
- **Stage**: RuntimeModuleStage::Attach
- **Generated Output**: Consumes loading handler + data
```javascript
__webpack_require__.consumesLoadingData = { chunkMapping, moduleIdToConsumeDataMapping, initialConsumes };
__webpack_require__.f.consumes = function(chunkId, promises) { ... };
```
- **Added When**: ConsumeSharedPlugin is active (always adds to tree)
- **Additional Code**: Includes consumesCommon.js, consumesInitial.js, consumesLoading.js based on configuration

#### 4. **ShareRuntimeModule**
- **File**: `share_runtime_module.rs`
- **Purpose**: Initializes shared scope
- **Stage**: RuntimeModuleStage::Normal (implied)
- **Generated Output**: Shared scope initialization
```javascript
__webpack_require__.S = {};
__webpack_require__.initializeSharingData = { scopeToSharingDataMapping, uniqueName };
__webpack_require__.I = function() { ... };  // or implementation from initializeSharing.js
```
- **Added When**: SHARE_SCOPE_MAP requirement present (by ShareRuntimePlugin)

#### 5. **ExposeRuntimeModule**
- **File**: `expose_runtime_module.rs`
- **Purpose**: Sets up container initialization data
- **Stage**: RuntimeModuleStage::Attach
- **Generated Output**: Container and expose initialization
```javascript
__webpack_require__.initializeExposesData = {
  moduleMap: {},
  shareScope: "default"
};
__webpack_require__.getContainer = function() { ... };
__webpack_require__.initContainer = function() { ... };
```
- **Added When**: CURRENT_REMOTE_GET_SCOPE required (by ContainerPlugin, only if enhanced)

#### 6. **EmbedFederationRuntimeModule**
- **File**: `embed_federation_runtime_module.rs`
- **Purpose**: **CRITICAL** - Wraps startup to execute federation runtime modules first
- **Stage**: RuntimeModuleStage::from(11) - After RemoteRuntimeModule
- **Generated Output**: prevStartup wrapper pattern
```javascript
var prevStartup = __webpack_require__.x;  // or .X
var hasRun = false;
__webpack_require__.x = function() {
  if (\!hasRun) {
    hasRun = true;
    __webpack_require__(fed_module_1);
    __webpack_require__(fed_module_2);
  }
  if (typeof prevStartup === 'function') {
    return prevStartup();
  } else {
    console.warn('[MF] Invalid prevStartup');
  }
};
```
- **Added When**: Runtime chunks with federation dependencies (by EmbedFederationRuntimePlugin)
- **Key Function**: Ensures federation code runs before other startup code

### Runtime Module Dependency Graph

```
Entry Chunk Startup
     ↓
EmbedFederationRuntimeModule (Stage 11)
     ├─ Executes federation runtime dependencies
     ├─ Wraps __webpack_require__.x or .X
     └─ Calls prevStartup()
         ↓
     FederationDataRuntimeModule (Stage Normal)
         ├─ Provides federation data
         └─ Used by other federation modules
         
     RemoteRuntimeModule (Stage 10)
         ├─ Registers __webpack_require__.f.remotes
         ├─ Loaded when chunk needs remotes
         └─ Uses FederationDataRuntimeModule
         
     ConsumeSharedRuntimeModule (Stage Attach)
         ├─ Registers __webpack_require__.f.consumes
         ├─ Provides consumesLoadingData
         └─ May include consumesCommon.js, consumesInitial.js, consumesLoading.js
         
     ShareRuntimeModule (Stage Normal)
         ├─ Registers __webpack_require__.I (initializeSharing)
         ├─ Provides initializeSharingData
         └─ Used by both ConsumeSharedRuntimeModule and RemoteRuntimeModule
         
     ExposeRuntimeModule (Stage Attach)
         ├─ Container initialization
         └─ Only added if enhanced mode
```

---

## 5. No Custom "federation-entry-startup" Requirement Exists

Currently, there is **NO custom runtime requirement** like "federation-entry-startup" or similar.

Instead, federation uses:
- **STARTUP_ENTRYPOINT** (when `mf_async_startup = true`)
- **STARTUP** (when `mf_async_startup = false`)

### Why Not a Custom Requirement?

1. **Existing Infrastructure**: STARTUP/STARTUP_ENTRYPOINT already exist
2. **Minimal Overhead**: EmbedFederationRuntimePlugin reuses existing wrapping
3. **Flexibility**: Works with both sync and async modes

### Could We Add One?

**YES**, but would need to:
1. Add new RuntimeGlobal constant (e.g., `FEDERATION_STARTUP`)
2. Implement separate startup template
3. Modify multiple plugins to recognize it
4. Would be **redundant** unless adding special handling like Promise.all wrapping

---

## 6. How EmbedFederationRuntimeModule Interacts with Startup

### Current Interaction Flow

```
1. ModuleFederationRuntimePlugin::additional_tree_runtime_requirements()
   ├─ Adds FederationDataRuntimeModule
   └─ If mf_async_startup: adds STARTUP_ENTRYPOINT, ENSURE_CHUNK, ENSURE_CHUNK_INCLUDE_ENTRIES

2. EmbedFederationRuntimePlugin::compilation()
   ├─ Registers AddFederationRuntimeDependencyHook listener
   └─ Taps JavascriptModulesRenderStartup hook

3. EmbedFederationRuntimePlugin::runtime_requirement_in_tree()
   ├─ For runtime chunks only: adds EmbedFederationRuntimeModule
   └─ Passes collected federation dependency IDs

4. EmbedFederationRuntimePlugin::render_startup()
   ├─ For entry chunks with no runtime:
   │  └─ Appends startup call: __webpack_require__.X(); or __webpack_require__.x();
   └─ For runtime+entry chunks: No action (JavaScript plugin handles it)

5. EmbedFederationRuntimeModule::generate()
   ├─ Creates prevStartup wrapper
   ├─ Wraps __webpack_require__.x or .X
   └─ Executes federation modules synchronously inside wrapper
```

### Key Interaction Points

#### A. render_startup Hook

**File**: `/crates/rspack_plugin_mf/src/container/embed_federation_runtime_plugin.rs:163-225`

The `render_startup` hook intercepts startup rendering:

```rust
#[plugin_hook(JavascriptModulesRenderStartup for EmbedFederationRuntimePlugin)]
async fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _module: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  let has_runtime = chunk.has_runtime(&compilation.chunk_group_by_ukey);
  let has_entry_modules = compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0;

  // Runtime chunks with entry modules: JavaScript plugin handles startup naturally
  if has_runtime && has_entry_modules {
    return Ok(());
  }

  // Entry chunks delegating to runtime need explicit startup calls
  if \!has_runtime && has_entry_modules {
    let mut startup_with_call = ConcatSource::default();
    
    let startup_global = if compilation.options.experiments.mf_async_startup {
      RuntimeGlobals::STARTUP_ENTRYPOINT
    } else {
      RuntimeGlobals::STARTUP
    };

    startup_with_call.add(RawStringSource::from("\n// Federation startup call\n"));
    startup_with_call.add(RawStringSource::from(format\!(
      "{}();\n",
      startup_global.name()
    )));

    startup_with_call.add(render_source.source.clone());
    render_source.source = startup_with_call.boxed();
  }

  Ok(())
}
```

**Key Logic**:
- If chunk has both runtime AND entry modules: Let JavaScript plugin handle it normally
- If chunk has ONLY entry modules: Explicitly call startup function at the end
- Chooses STARTUP_ENTRYPOINT or STARTUP based on mf_async_startup flag

#### B. Startup Wrapper Generation

The generated wrapper in EmbedFederationRuntimeModule:

```rust
// Generate prevStartup wrapper pattern with defensive checks
// Use STARTUP_ENTRYPOINT when mf_async_startup is enabled, otherwise use STARTUP
let startup = if compilation.options.experiments.mf_async_startup {
  RuntimeGlobals::STARTUP_ENTRYPOINT.name()  // "__webpack_require__.X"
} else {
  RuntimeGlobals::STARTUP.name()  // "__webpack_require__.x"
};

let result = format\!(
  r#"var prevStartup = {startup};
var hasRun = false;
{startup} = function() {{
	if (\!hasRun) {{
		hasRun = true;
{module_executions}
	}}
	if (typeof prevStartup === 'function') {{
		return prevStartup();
	}} else {{
		console.warn('[MF] Invalid prevStartup');
	}}
}};"#
);
```

---

## 7. Can We Add Custom Runtime Requirement for Federation Startup?

### YES - Implementation Path

We could add a custom runtime requirement to enable Promise.all wrapping. Here's the approach:

#### Option A: Create FederationStartupHandler Requirement

1. **Add New RuntimeGlobal**

```rust
// In rspack_core/src/runtime_globals.rs
pub const FEDERATION_STARTUP: u64 = 1 << 47;  // New bit
```

2. **Create New Runtime Module Template**

```rust
// New file: federation_startup_handler.rs
#[impl_runtime_module]
pub struct FederationStartupHandlerModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl RuntimeModule for FederationStartupHandlerModule {
  async fn generate(&self, compilation: &Compilation) -> Result<String> {
    // Check if any federation dependencies return Promises
    // Generate Promise.all wrapper if needed
    let has_promise_deps = /* check collected deps */;
    
    if has_promise_deps {
      return Ok(r#"
__webpack_require__.federationStartup = function(deps) {
  return Promise.all(deps.map(function(id) {
    return __webpack_require__.e(id);
  }));
};
"#.to_string());
    }
    Ok("".to_string())
  }
}
```

3. **Modify EmbedFederationRuntimePlugin**

```rust
// In embed_federation_runtime_plugin.rs
async fn runtime_requirement_in_tree(...) -> Result<Option<()>> {
  if has_runtime && has_federation_deps {
    // Add FEDERATION_STARTUP requirement
    runtime_requirements_mut.insert(RuntimeGlobals::FEDERATION_STARTUP);
    
    // Add the handler module
    compilation.add_runtime_module(
      chunk_ukey,
      Box::new(FederationStartupHandlerModule::new()),
    )?;
  }
  Ok(None)
}
```

4. **Modify EmbedFederationRuntimeModule**

```rust
async fn generate(&self, compilation: &Compilation) -> Result<String> {
  // Check if FEDERATION_STARTUP is required
  let chunk_runtime_requirements = 
    ChunkGraph::get_chunk_runtime_requirements(compilation, &chunk_ukey);
  
  let use_federation_startup = chunk_runtime_requirements
    .contains(RuntimeGlobals::FEDERATION_STARTUP);
  
  if use_federation_startup {
    // Generate Promise.all wrapper
    let deps_to_load: Vec<_> = collected_deps.iter()
      .filter(|dep| /* is async */)
      .map(|dep| format\!("__webpack_require__.e({})", dep))
      .collect();
    
    return Ok(format\!(
      r#"var prevStartup = {startup};
__webpack_require__.{startup} = function() {{
  if (\!hasRun) {{
    hasRun = true;
    return Promise.all([{deps}]).then(function() {{
      {module_executions}
      return typeof prevStartup === 'function' ? prevStartup() : undefined;
    }});
  }}
  return typeof prevStartup === 'function' ? prevStartup() : undefined;
}};"#
    ));
  }
  
  // Otherwise: current synchronous wrapper
}
```

#### Option B: Extend STARTUP_ENTRYPOINT Behavior

Simpler approach - modify existing templates:

1. **Check for federation deps in existing startup template**
2. **If present, wrap in Promise.all**
3. **No new runtime requirement needed**

This is the **recommended approach** because:
- Minimal changes
- No new runtime infrastructure
- Uses existing async capability of STARTUP_ENTRYPOINT
- Already has Promise support

---

## 8. Recommendations for Promise.all Wrapping Logic

### A. Detection Points

Federation dependencies should be wrapped in Promise.all if:

1. **Module is async** (returns Promise)
2. **Module depends on chunks** (remote loading, sharing init)
3. **Multiple dependencies exist** (multiple remotes, multiple shared modules)

### B. Where to Add Promise.all Wrapping

#### Primary Location: EmbedFederationRuntimeModule

**File**: `/crates/rspack_plugin_mf/src/container/embed_federation_runtime_module.rs`

**Current Logic** (Lines 75-84):
```rust
// Generate module execution code for each federation runtime dependency
let mut runtime_requirements = RuntimeGlobals::default();
let mut module_executions = String::with_capacity(federation_runtime_modules.len() * 50);

for dep_id in federation_runtime_modules {
  let module_str = module_raw(compilation, &mut runtime_requirements, &dep_id, "", false);
  module_executions.push_str("\t\t");
  module_executions.push_str(&module_str);
  module_executions.push('\n');
}
```

**Recommended Modification**:
```rust
// Check if any federation modules are async or depend on chunks
let mut has_promise_modules = false;
let mut promises_vec = Vec::new();

for dep_id in federation_runtime_modules.iter() {
  let module_dyn = module_graph.get_module_by_dependency_id(&dep_id)
    .expect("Module should exist");
  
  // Check if module depends on chunks (requires ENSURE_CHUNK)
  let has_chunk_deps = compilation
    .chunk_graph
    .get_chunk_dependencies(&chunk_ukey)
    .count() > 0;
  
  if has_chunk_deps {
    has_promise_modules = true;
    // Store as promise loading expression
    promises_vec.push(format\!("__webpack_require__.e({})", /* chunk_id */));
  }
}

// Generate appropriate wrapper based on whether promises are needed
if has_promise_modules && compilation.options.experiments.mf_async_startup {
  // Use Promise.all wrapper
  generate_promise_wrapped_startup(promises_vec, module_executions)
} else {
  // Use synchronous wrapper (current implementation)
  generate_sync_wrapper(module_executions)
}
```

#### Secondary Location: ModuleFederationRuntimePlugin

**File**: `/crates/rspack_plugin_mf/src/container/module_federation_runtime_plugin.rs`

Could determine earlier whether federation startup needs async handling:

```rust
#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ModuleFederationRuntimePlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  compilation.add_runtime_module(chunk_ukey, Box::<FederationDataRuntimeModule>::default())?;

  if compilation.options.experiments.mf_async_startup {
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

    if chunk.has_runtime(&compilation.chunk_group_by_ukey)
      && compilation
        .chunk_graph
        .get_number_of_entry_modules(chunk_ukey)
        > 0
    {
      runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
      
      // NEW: Check if federation loading requires async wrapping
      let hooks = FederationModulesPlugin::get_compilation_hooks(compilation);
      let federation_deps = hooks.collected_federation_runtime_dependency_ids.lock().unwrap();
      
      // If federation deps exist and any depend on chunks, mark for async
      if \!federation_deps.is_empty() {
        runtime_requirements.insert(RuntimeGlobals::FEDERATION_ASYNC_STARTUP); // NEW
      }
    }
  }

  Ok(())
}
```

### C. Generated Code Examples

#### Current (Synchronous) Wrapper
```javascript
var prevStartup = __webpack_require__.x;
var hasRun = false;
__webpack_require__.x = function() {
  if (\!hasRun) {
    hasRun = true;
    __webpack_require__(123);  // Federation module 1
    __webpack_require__(124);  // Federation module 2
  }
  if (typeof prevStartup === 'function') {
    return prevStartup();
  }
};
```

#### Proposed (Async Promise.all) Wrapper
```javascript
var prevStartup = __webpack_require__.X;
var hasRun = false;
var fedPromise;
__webpack_require__.X = function() {
  if (\!hasRun) {
    hasRun = true;
    fedPromise = Promise.all([
      __webpack_require__.e(1),  // Ensure chunks needed by federation deps
      __webpack_require__.e(2)
    ]).then(function() {
      __webpack_require__(123);  // Federation module 1
      __webpack_require__(124);  // Federation module 2
      return typeof prevStartup === 'function' ? prevStartup() : undefined;
    });
    return fedPromise;
  }
  return fedPromise || (typeof prevStartup === 'function' ? prevStartup() : undefined);
};
```

### D. Integration Points

The Promise.all wrapping should integrate with:

1. **RemoteRuntimeModule handlers**
   - Already return promises
   - Handlers push to promises array in ensureChunk
   
2. **ConsumeSharedRuntimeModule handlers**
   - Already handle promises in resolveHandler()
   - Integrate with Promise.all chain

3. **Array-Push-Callback format**
   - Current ON_CHUNKS_LOADED: Two-phase (register, execute)
   - New approach: Single async phase with Promise.all

### E. Stages of Execution

```
1. Entry chunk loads
2. EmbedFederationRuntimeModule wraps __webpack_require__.X
3. Startup function called: __webpack_require__.X()
4. If async wrapping enabled:
   a. Promise.all loads federation dependency chunks
   b. Federation modules execute inside promise.then()
   c. prevStartup called after federation modules ready
   d. Promise returned to caller
5. If sync wrapping (current):
   a. Federation modules execute immediately
   b. prevStartup called synchronously
   c. Undefined or export returned
```

---

## Summary Table: All Federation Runtime Requirements

| Requirement | Added By | When | Purpose | Stages |
|-------------|----------|------|---------|--------|
| FederationDataRuntimeModule | ModuleFederationRuntimePlugin | Always | Base federation data | Normal |
| STARTUP_ENTRYPOINT | ModuleFederationRuntimePlugin, EmbedFederationRuntimePlugin | mf_async_startup=true | Async MF startup | Template-based |
| STARTUP | EmbedFederationRuntimePlugin | Default | Sync startup wrapper | Template-based |
| RemoteRuntimeModule | ContainerReferencePlugin | ENSURE_CHUNK_HANDLERS | Remotes loading | 10 |
| ConsumeSharedRuntimeModule | ConsumeSharedPlugin | Always if consume exists | Consumes resolution | Attach |
| ShareRuntimeModule | ShareRuntimePlugin | SHARE_SCOPE_MAP required | Scope initialization | Normal |
| ExposeRuntimeModule | ContainerPlugin | CURRENT_REMOTE_GET_SCOPE | Expose initialization | Attach |
| EmbedFederationRuntimeModule | EmbedFederationRuntimePlugin | Has federation deps | Federation startup wrap | 11 |
| MODULE | Multiple (Consume, Remote) | When federation used | Module object | - |
| MODULE_CACHE | ConsumeSharedPlugin | When consume exists | Cache management | - |
| MODULE_FACTORIES_ADD_ONLY | Multiple | When federation used | Factory functions | - |
| SHARE_SCOPE_MAP | ConsumeSharedPlugin, ContainerReferencePlugin | When federation used | Sharing scope map | - |
| INITIALIZE_SHARING | ConsumeSharedPlugin, ContainerReferencePlugin | When federation used | Init sharing | - |
| HAS_OWN_PROPERTY | Multiple | When federation used | Property checking | - |

---

## Conclusion

Module Federation runtime architecture in rspack is sophisticated:

1. **No custom federation-entry-startup requirement** - Uses existing STARTUP/STARTUP_ENTRYPOINT
2. **Consumes/Remotes handled via chunk handlers** - Registered in runtime modules, called during ensureChunk
3. **EmbedFederationRuntimeModule is KEY** - Wraps startup to execute federation modules first
4. **Promise.all wrapping is feasible** - Add detection in EmbedFederationRuntimeModule to wrap federation module execution when they depend on async chunk loading
5. **Integration points clear** - Can modify EmbedFederationRuntimeModule and/or ModuleFederationRuntimePlugin to add Promise.all support

The recommended approach is **Option B** - extend existing STARTUP_ENTRYPOINT behavior to wrap federation module execution in Promise.all when needed, rather than creating a new runtime requirement.

