# Comprehensive Research: generate_entry_startup Function in rspack_plugin_runtime

## Overview

The `generate_entry_startup` function in `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/helpers.rs` (lines 181-273) is a critical code generation utility that creates the JavaScript code responsible for executing entry modules. It handles the crucial link between rspack's module system and application startup.

---

## 1. Function Signature and Parameters

### Complete Function Signature
```rust
pub fn generate_entry_startup(
  compilation: &Compilation,
  chunk: &ChunkUkey,
  entries: &IdentifierLinkedMap<ChunkGroupUkey>,
  passive: bool,
) -> BoxSource
```

### Parameter Details

| Parameter | Type | Purpose |
|-----------|------|---------|
| `compilation` | `&Compilation` | Full compilation context containing module graph, chunk graph, module IDs, chunk IDs |
| `chunk` | `&ChunkUkey` | The unique key of the current chunk being processed |
| `entries` | `&IdentifierLinkedMap<ChunkGroupUkey>` | Map of entry module identifiers to their chunk group keys; this determines which modules are entry points |
| `passive` | `bool` | Controls runtime global selection: `true` = ON_CHUNKS_LOADED, `false` = STARTUP_ENTRYPOINT |

### Parameter Flow

```
┌─────────────────────────────────────────────────────────────┐
│ Input Parameters                                            │
├─────────────────────────────────────────────────────────────┤
│ compilation: Compilation                                    │
│   ├─ module_graph: resolves module identifiers to objects   │
│   ├─ module_ids_artifact: maps modules to numeric IDs       │
│   ├─ chunk_by_ukey: maps chunk keys to chunk objects        │
│   ├─ chunk_ids_artifact: maps chunks to string IDs          │
│   └─ chunk_group_by_ukey: entry point info                  │
│                                                              │
│ entries: IdentifierLinkedMap<ChunkGroupUkey>                │
│   ├─ Key: ModuleIdentifier (e.g., module path)             │
│   └─ Value: ChunkGroupUkey (chunk group containing entry)   │
│                                                              │
│ passive: bool (Set by caller)                               │
│   ├─ true: From ArrayPushCallbackChunkFormat when           │
│   │         experiments.mf_async_startup = false            │
│   └─ false: From CommonJsChunkFormat (always false)         │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. The `passive` Parameter - Impact on Generated Code

### How `passive` Controls Runtime Global Selection

The `passive` parameter is the primary control flag that determines which runtime method is used:

#### When `passive = true` (ON_CHUNKS_LOADED path)
- **Location**: Line 249-269
- **Runtime Global**: `RuntimeGlobals::ON_CHUNKS_LOADED` = `__webpack_require__.O`
- **Behavior**: Two-phase deferred execution
- **Context**: ArrayPushCallbackChunkFormat with `experiments.mf_async_startup = false`

**Generated Code Pattern**:
```javascript
// Phase 1: Register callback (returns undefined)
__webpack_require__.O(0, [chunk_ids], function() {
  return __webpack_exec__(module_id);
});

// Phase 2: Execute deferred queue
var __webpack_exports__ = __webpack_require__.O();
```

#### When `passive = false` (STARTUP_ENTRYPOINT path)
- **Location**: Line 249-259
- **Runtime Global**: `RuntimeGlobals::STARTUP_ENTRYPOINT` = `__webpack_require__.X`
- **Behavior**: Direct async execution (can return Promise)
- **Context**: CommonJsChunkFormat (always false)

**Generated Code Pattern**:
```javascript
var __webpack_exports__ = (module_id_exprs);
// OR if chunks present:
var __webpack_exports__ = __webpack_require__.X(0, [chunks], function() {
  return module_id_exprs;
});
```

### Code Location Showing `passive` Impact

**Lines 249-270 in helpers.rs**:
```rust
} else {
  if \!passive {
    source.push_str("var __webpack_exports__ = ");
  }
  source.push_str(&format\!(
    "{}(0, {}, function() {{
      return {};
    }});\n",
    if passive {
      RuntimeGlobals::ON_CHUNKS_LOADED
    } else {
      RuntimeGlobals::STARTUP_ENTRYPOINT
    },
    stringify_chunks_to_array(&chunks_ids),
    module_ids_code
  ));
  if passive {
    source.push_str(&format\!(
      "var __webpack_exports__ = {}();\n",
      RuntimeGlobals::ON_CHUNKS_LOADED
    ));
  }
}
```

### Key Insight: Two-Phase vs Direct

- **passive=true (ON_CHUNKS_LOADED)**: Requires TWO calls
  1. First call: Register the callback (returns undefined)
  2. Second call: Execute the queue and get exports
  
- **passive=false (STARTUP_ENTRYPOINT)**: Single direct call with deferred execution via callback

---

## 3. Runtime Globals Used

### Complete Runtime Global Reference

| Global | Bit Position | JavaScript Name | Purpose | Line in Code |
|--------|--------------|-----------------|---------|------|
| `ON_CHUNKS_LOADED` | 1 << 15 | `__webpack_require__.O` | Deferred chunk loading callback queue | 257 |
| `STARTUP_ENTRYPOINT` | 1 << 34 | `__webpack_require__.X` | Async entry point executor | 259 |
| `ENTRY_MODULE_ID` | 1 << 39 | `__webpack_require__.s` | Stores the entry module ID | 234 |
| `REQUIRE` | 1 << 5 | `__webpack_require__` | Module require function | 233 |

### How These Globals Are Used

#### ENTRY_MODULE_ID Usage (Line 233-235)
```rust
source.push_str(&format\!(
  "var __webpack_exec__ = function(moduleId) {{ return __webpack_require__({} = moduleId) }}\n",
  RuntimeGlobals::ENTRY_MODULE_ID
));
```

**Generated JavaScript**:
```javascript
var __webpack_exec__ = function(moduleId) { 
  return __webpack_require__(__webpack_require__.s = moduleId) 
}
```

**Purpose**: Creates a wrapper function that:
1. Sets the entry module ID globally (`__webpack_require__.s = moduleId`)
2. Requires the module
3. Returns the module's exports

This allows the module execution context to know which module is the entry point.

#### ON_CHUNKS_LOADED vs STARTUP_ENTRYPOINT

**ON_CHUNKS_LOADED Signature** (from runtime definition):
```
__webpack_require__.O(priority, chunkIds, callback) -> any
```
- Registers a callback to execute when chunks load
- Second call with no args processes queued callbacks
- Returns undefined on first call, exports on second call

**STARTUP_ENTRYPOINT Signature** (from runtime definition):
```
__webpack_require__.X(priority, chunkIds, callback) -> Promise | any
```
- Direct async execution
- Can return Promise if async template is used
- Single call pattern (no queue)

---

## 4. Where This Function Is Called From

### Call Site 1: ArrayPushCallbackChunkFormatPlugin (array_push_callback_chunk_format.rs:156)

**File**: `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/array_push_callback_chunk_format.rs`
**Lines**: 151-175
**Context**: Renders chunks for array-push-callback format (JSONP-style)

```rust
let passive = \!compilation.options.experiments.mf_async_startup;
let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, passive);
```

**Caller Context**:
- `passive=true` when `mf_async_startup=false` (default, uses ON_CHUNKS_LOADED)
- `passive=false` when `mf_async_startup=true` (async Module Federation, uses STARTUP_ENTRYPOINT)

**Caller Flow**:
```
ArrayPushCallbackChunkFormatPlugin::render_chunk
  ├─ Check if chunk has entry modules (line 151)
  ├─ Create entries map (line 152-154)
  ├─ Determine passive flag (line 155)
  ├─ Call generate_entry_startup (line 156)
  ├─ Call render_startup hook for extensions (line 164-174)
  └─ Embed startup code in callback function
```

### Call Site 2: CommonJsChunkFormatPlugin (common_js_chunk_format.rs:155)

**File**: `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/common_js_chunk_format.rs`
**Lines**: 152-174
**Context**: Renders chunks for CommonJS format (Node.js-style exports)

```rust
let entries = compilation
  .chunk_graph
  .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);
let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, false);
```

**Key Difference**: Always passes `passive=false`
- Always uses STARTUP_ENTRYPOINT for CommonJS chunks
- No mf_async_startup flag check

**Caller Flow**:
```
CommonJsChunkFormatPlugin::render_chunk
  ├─ Check if chunk has entry module (line 135)
  ├─ Generate startup code (line 155)
  ├─ Call render_startup hook for extensions (line 164-174)
  └─ Wrap in IIFE and return
```

### Call Sites Summary

| Plugin | File | Line | passive Value | Runtime Global |
|--------|------|------|---------------|-----------------|
| ArrayPushCallbackChunkFormatPlugin | array_push_callback_chunk_format.rs | 156 | `\!mf_async_startup` | ON_CHUNKS_LOADED or STARTUP_ENTRYPOINT |
| CommonJsChunkFormatPlugin | common_js_chunk_format.rs | 155 | `false` | STARTUP_ENTRYPOINT |

---

## 5. Structure of Generated Code

### Step-by-Step Generation Process

The function generates startup code in 4 main phases:

#### Phase 1: Create __webpack_exec__ Wrapper (Lines 231-235)

```rust
let mut source = String::default();
source.push_str(&format\!(
  "var __webpack_exec__ = function(moduleId) {{ return __webpack_require__({} = moduleId) }}\n",
  RuntimeGlobals::ENTRY_MODULE_ID
));
```

**Purpose**: Creates a helper function that:
1. Sets the entry module ID in `__webpack_require__.s`
2. Requires the module
3. Returns module exports

**Generated JavaScript**:
```javascript
var __webpack_exec__ = function(moduleId) { 
  return __webpack_require__(__webpack_require__.s = moduleId) 
}
```

#### Phase 2: Build Module ID Expressions (Lines 187-229)

```rust
let mut module_id_exprs = vec\![];
let mut chunks_ids = HashSet::default();
let module_graph = compilation.get_module_graph();

for (module, entry) in entries {
  // 1. Get module object from module graph
  if let Some(module_id) = module_graph
    .module_by_identifier(module)
    .filter(|module| {
      module.source_types(&module_graph).contains(&SourceType::JavaScript)
    })
    .and_then(|module| {
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier())
    })
  {
    // 2. Convert module ID to JSON string (e.g., "123" or "\"chunk-name\"")
    let module_id_expr = serde_json::to_string(module_id).expect("invalid module_id");
    module_id_exprs.push(module_id_expr);
  }

  // 3. Collect all chunks needed by this entry
  if let Some(runtime_chunk) = compilation
    .chunk_group_by_ukey
    .get(entry)
    .map(|e| e.get_runtime_chunk(&compilation.chunk_group_by_ukey))
  {
    let chunks = get_all_chunks(entry, chunk, Some(&runtime_chunk), ...);
    chunks_ids.extend(chunks.iter().map(|chunk_ukey| {
      let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
      chunk.expect_id(&compilation.chunk_ids_artifact).clone()
    }));
  }
}

// 4. Create comma-separated list of __webpack_exec__ calls
let module_ids_code = &module_id_exprs
  .iter()
  .map(|module_id_expr| format\!("__webpack_exec__({module_id_expr})"))
  .collect::<Vec<_>>()
  .join(", ");
```

**Example**:
- Input: 2 entry modules with IDs `0` and `1`
- Output: `module_ids_code = "__webpack_exec__(0), __webpack_exec__(1)"`
- `chunks_ids = {2, 3, 4}` (dependent chunk IDs)

#### Phase 3: Decide on Simple vs Complex Path (Lines 242-270)

**Simple Path** (No dependent chunks):
```rust
if chunks_ids.is_empty() {
  if \!module_ids_code.is_empty() {
    source.push_str("var __webpack_exports__ = (");
    source.push_str(module_ids_code);
    source.push_str(");\n");
  }
}
```

**Generated JavaScript**:
```javascript
var __webpack_exports__ = (__webpack_exec__(0));
```

**Complex Path** (Has dependent chunks):
```rust
else {
  if \!passive {
    source.push_str("var __webpack_exports__ = ");
  }
  source.push_str(&format\!(
    "{}(0, {}, function() {{
      return {};
    }});\n",
    if passive {
      RuntimeGlobals::ON_CHUNKS_LOADED
    } else {
      RuntimeGlobals::STARTUP_ENTRYPOINT
    },
    stringify_chunks_to_array(&chunks_ids),
    module_ids_code
  ));
  if passive {
    source.push_str(&format\!(
      "var __webpack_exports__ = {}();\n",
      RuntimeGlobals::ON_CHUNKS_LOADED
    ));
  }
}
```

**Generated JavaScript (passive=true)**:
```javascript
__webpack_require__.O(0, [2, 3, 4], function() {
  return __webpack_exec__(0), __webpack_exec__(1);
});
var __webpack_exports__ = __webpack_require__.O();
```

**Generated JavaScript (passive=false)**:
```javascript
var __webpack_exports__ = __webpack_require__.X(0, [2, 3, 4], function() {
  return __webpack_exec__(0), __webpack_exec__(1);
});
```

#### Phase 4: Return as BoxSource (Line 272)

```rust
RawStringSource::from(source).boxed()
```

Converts the String to a `RawStringSource` and boxes it for use in the source concatenation system.

### Complete Code Generation Example

**Input**:
```
entries = [
  ("module-0" -> ChunkGroup containing chunks [runtime])
]
chunks_ids = [1, 2, 3]
passive = true
```

**Generated Output**:
```javascript
var __webpack_exec__ = function(moduleId) { 
  return __webpack_require__(__webpack_require__.s = moduleId) 
}
__webpack_require__.O(0, [1, 2, 3], function() {
  return __webpack_exec__(0);
});
var __webpack_exports__ = __webpack_require__.O();
```

---

## 6. Extension and Injection Points

### Hook-Based Extension System

The generated startup code is passed through a **hook system** that allows plugins to inject custom code before/after entry execution.

#### Hook Definition (drive.rs:13)

```rust
define_hook\!(JavascriptModulesRenderStartup: 
  Series(
    compilation: &Compilation, 
    chunk_ukey: &ChunkUkey, 
    module: &ModuleIdentifier, 
    source: &mut RenderSource
  )
);
```

**Hook Characteristics**:
- **Type**: Series hook (all handlers execute, all modifications apply)
- **Parameters**:
  - `compilation`: Full compilation context
  - `chunk_ukey`: Current chunk being rendered
  - `module`: The last entry module identifier
  - `source`: Mutable RenderSource containing generated startup code
- **Effect**: Handlers can modify the source in-place

#### How Hooks Are Called (array_push_callback_chunk_format.rs:164-174)

```rust
let last_entry_module = entries
  .keys()
  .next_back()
  .expect("should have last entry module");

let mut render_source = RenderSource {
  source: start_up_source,  // Generated by generate_entry_startup
};

hooks
  .try_read()
  .expect("should have js plugin drive")
  .render_startup
  .call(
    compilation,
    chunk_ukey,
    last_entry_module,
    &mut render_source,
  )
  .await?;

source.add(render_source.source);  // Add potentially modified source
```

#### Using the Hook for Injection

Plugins implement the `JavascriptModulesRenderStartup` hook to inject code.

**Example: AssignLibraryPlugin (assign_library_plugin.rs:245-268)**

```rust
#[plugin_hook(JavascriptModulesRenderStartup for AssignLibraryPlugin)]
async fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  // Check if this chunk should be processed
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };

  // Clone existing source
  let mut source = ConcatSource::default();
  source.add(render_source.source.clone());
  
  // ... compute export assignment code ...
  
  // Append assignment code to source
  source.add(RawStringSource::from(
    format\!("{} = __webpack_exports__;", full_name_str)
  ));
  
  // Replace the source
  render_source.source = source.boxed();
  Ok(())
}
```

**Flow**:
```
1. generate_entry_startup() produces:
   var __webpack_exports__ = __webpack_exec__(0);

2. render_startup hook runs:
   AssignLibraryPlugin sees this chunk needs library export

3. Plugin appends:
   window.myLibrary = __webpack_exports__;

4. Final result:
   var __webpack_exports__ = __webpack_exec__(0);
   window.myLibrary = __webpack_exports__;
```

### Available Hook Implementations

The following plugins tap into `render_startup` for code injection:

| Plugin | Purpose | File |
|--------|---------|------|
| AssignLibraryPlugin | Assign exports to window/object property | assign_library_plugin.rs |
| ExportPropertyLibraryPlugin | Export specific properties | export_property_library_plugin.rs |
| ModuleLibraryPlugin | Module.exports compatibility | module_library_plugin.rs |
| ModernModuleLibraryPlugin | Modern module library format | modern_module_library_plugin.rs |
| ModuleChunkFormatPlugin | Module chunk format | runtime/module_chunk_format.rs |
| EmbedFederationRuntimePlugin | Module Federation runtime | mf/embed_federation_runtime_plugin.rs |

### How to Create a Custom Injection Plugin

```rust
#[plugin_hook(JavascriptModulesRenderStartup for MyCustomPlugin)]
async fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  // Get chunk and check if we should process it
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  
  // Option 1: Wrap existing code
  let mut wrapper = ConcatSource::default();
  wrapper.add(RawStringSource::from("console.log('startup');"));
  wrapper.add(render_source.source.clone());
  wrapper.add(RawStringSource::from("console.log('done');"));
  render_source.source = wrapper.boxed();
  
  // Option 2: Replace entirely
  render_source.source = RawStringSource::from(
    "// Custom startup code".to_string()
  ).boxed();
  
  Ok(())
}
```

---

## 7. Key Code Sections - Detailed Breakdown

### Section 1: Module ID Extraction (Lines 190-206)

**Purpose**: Extract the numeric/string ID for each entry module

```rust
for (module, entry) in entries {
  if let Some(module_id) = module_graph
    .module_by_identifier(module)                    // Get Module object
    .filter(|module| {                               // Filter to JS modules
      module
        .source_types(&module_graph)
        .contains(&SourceType::JavaScript)
    })
    .and_then(|module| {                             // Get module ID
      ChunkGraph::get_module_id(
        &compilation.module_ids_artifact, 
        module.identifier()
      )
    })
  {
    let module_id_expr = serde_json::to_string(module_id)
      .expect("invalid module_id");                  // Convert to JSON
    module_id_exprs.push(module_id_expr);
  } else {
    continue;                                        // Skip non-JS modules
  }
```

**Key Logic**:
1. Iterate through all entry modules
2. For each module, get its Module object from the module graph
3. Check that it's a JavaScript source type (skip CSS, assets, etc.)
4. Get the assigned numeric ID (e.g., `0`, `1`) or string ID (e.g., `"chunk-name"`)
5. Convert to JSON format for inclusion in JavaScript code
6. Store in `module_id_exprs` vector

### Section 2: Chunk Dependency Collection (Lines 208-228)

**Purpose**: Find all chunks that must be loaded before executing the entry module

```rust
if let Some(runtime_chunk) = compilation
  .chunk_group_by_ukey
  .get(entry)                                        // Get chunk group
  .map(|e| e.get_runtime_chunk(&compilation.chunk_group_by_ukey))
{
  let chunks = get_all_chunks(
    entry,                                           // Entry chunk group
    chunk,                                           // Exclude current chunk
    Some(&runtime_chunk),                            // Exclude runtime chunk
    &compilation.chunk_group_by_ukey,
  );
  chunks_ids.extend(
    chunks
      .iter()
      .map(|chunk_ukey| {
        let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
        chunk.expect_id(&compilation.chunk_ids_artifact).clone()
      })
      .collect::<HashSet<_>>(),
  );
}
```

**Key Logic**:
1. Get the chunk group for this entry
2. Find the runtime chunk
3. Get all ancestor chunks needed for this entry
4. Exclude the current chunk and runtime chunk
5. Convert chunk keys to chunk IDs
6. Add to global `chunks_ids` set

**Result**: `chunks_ids` contains the IDs of all chunks that must load before entry execution

### Section 3: Runtime Global Selection (Lines 249-269)

**Purpose**: Generate different code based on `passive` flag and chunk dependencies

```rust
if chunks_ids.is_empty() {
  // Path A: No dependencies - simple execution
  if \!module_ids_code.is_empty() {
    source.push_str("var __webpack_exports__ = (");
    source.push_str(module_ids_code);
    source.push_str(");\n");
  }
} else {
  // Path B: Has dependencies - need chunk loading
  if \!passive {
    source.push_str("var __webpack_exports__ = ");    // Async path
  }
  source.push_str(&format\!(
    "{}(0, {}, function() {{
      return {};
    }});\n",
    if passive {
      RuntimeGlobals::ON_CHUNKS_LOADED   // .O
    } else {
      RuntimeGlobals::STARTUP_ENTRYPOINT // .X
    },
    stringify_chunks_to_array(&chunks_ids),
    module_ids_code
  ));
  if passive {
    // Two-phase: queue + execute
    source.push_str(&format\!(
      "var __webpack_exports__ = {}();\n",
      RuntimeGlobals::ON_CHUNKS_LOADED
    ));
  }
}
```

**Decision Tree**:
```
Has dependencies (chunks_ids non-empty)?
├─ NO (empty)
│  └─ Simple: var __webpack_exports__ = (__webpack_exec__(0));
│
└─ YES (non-empty)
   ├─ passive=false (STARTUP_ENTRYPOINT)
   │  └─ Async: var __webpack_exports__ = __webpack_require__.X(0, [ids], fn);
   │
   └─ passive=true (ON_CHUNKS_LOADED)
      └─ Deferred:
         __webpack_require__.O(0, [ids], fn);
         var __webpack_exports__ = __webpack_require__.O();
```

---

## Summary

### Function Purpose
Generate JavaScript code that properly executes entry modules, handling:
- Entry module identification
- Chunk dependency resolution
- Runtime global selection (ON_CHUNKS_LOADED vs STARTUP_ENTRYPOINT)
- Module execution via __webpack_exec__ wrapper

### Critical Parameters
- `passive`: Controls STARTUP_ENTRYPOINT (false) vs ON_CHUNKS_LOADED (true)
- `entries`: Maps entry module identifiers to their chunk groups
- `compilation`: Provides access to module/chunk metadata

### Runtime Globals
- **ENTRY_MODULE_ID** (.s): Stores which module is the entry
- **ON_CHUNKS_LOADED** (.O): Deferred callback queue (passive mode)
- **STARTUP_ENTRYPOINT** (.X): Async entry executor (active mode)

### Extension Points
- **JavascriptModulesRenderStartup hook**: Allows plugins to inject code before/after startup
- Implemented by library plugins, Module Federation, and custom plugins

### Code Flow
```
generate_entry_startup()
├─ Extract entry module IDs
├─ Collect dependent chunk IDs
├─ Create __webpack_exec__ wrapper
├─ Generate startup code (simple or complex)
└─ Return as BoxSource

[Returned source] → render_startup hooks → [Modified source] → Final output
```

