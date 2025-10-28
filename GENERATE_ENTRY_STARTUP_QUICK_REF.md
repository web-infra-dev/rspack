# Quick Reference: generate_entry_startup Function

## File Location
**Path**: `/Users/zackjackson/rspack/crates/rspack_plugin_runtime/src/helpers.rs`  
**Lines**: 181-273

## Function Signature
```rust
pub fn generate_entry_startup(
  compilation: &Compilation,
  chunk: &ChunkUkey,
  entries: &IdentifierLinkedMap<ChunkGroupUkey>,
  passive: bool,
) -> BoxSource
```

## Parameters Quick Reference

| Param | Type | Controls |
|-------|------|----------|
| `compilation` | &Compilation | Access to module/chunk metadata, IDs |
| `chunk` | &ChunkUkey | Current chunk being processed |
| `entries` | IdentifierLinkedMap | Maps module identifiers to chunk groups |
| `passive` | bool | **ON_CHUNKS_LOADED (true) vs STARTUP_ENTRYPOINT (false)** |

## passive Parameter Effect

```
passive = true  → __webpack_require__.O (ON_CHUNKS_LOADED)
                   Two-phase: register + execute
                   From: ArrayPushCallbackChunkFormat when !mf_async_startup

passive = false → __webpack_require__.X (STARTUP_ENTRYPOINT)  
                   Single call with callback
                   From: CommonJsChunkFormat (always false)
```

## Code Generation Phases

### Phase 1: __webpack_exec__ Wrapper (Line 231-235)
```rust
var __webpack_exec__ = function(moduleId) { 
  return __webpack_require__(__webpack_require__.s = moduleId) 
}
```

### Phase 2: Extract Module IDs (Lines 190-206)
- Iterate entries
- Get module ID for each (numeric or string)
- Filter to JavaScript modules only
- Store in `module_id_exprs` vector

### Phase 3: Collect Chunk Dependencies (Lines 208-228)
- Get chunk group for each entry
- Use `get_all_chunks()` to find dependencies
- Exclude current chunk and runtime chunk
- Store chunk IDs in `chunks_ids` set

### Phase 4: Generate Code (Lines 242-270)

**If NO dependencies** (empty chunks_ids):
```rust
var __webpack_exports__ = (__webpack_exec__(0));
```

**If HAS dependencies** (non-empty chunks_ids):

When `passive=true`:
```javascript
__webpack_require__.O(0, [1,2,3], function() {
  return __webpack_exec__(0);
});
var __webpack_exports__ = __webpack_require__.O();
```

When `passive=false`:
```javascript
var __webpack_exports__ = __webpack_require__.X(0, [1,2,3], function() {
  return __webpack_exec__(0);
});
```

## Runtime Globals Used

| Global | JavaScript | Purpose |
|--------|-----------|---------|
| ENTRY_MODULE_ID | `__webpack_require__.s` | Stores entry module ID |
| ON_CHUNKS_LOADED | `__webpack_require__.O` | Deferred callback queue |
| STARTUP_ENTRYPOINT | `__webpack_require__.X` | Async entry executor |

## Where It's Called

### Call 1: ArrayPushCallbackChunkFormatPlugin (line 156)
```rust
let passive = !compilation.options.experiments.mf_async_startup;
let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, passive);
```
File: `crates/rspack_plugin_runtime/src/array_push_callback_chunk_format.rs`

### Call 2: CommonJsChunkFormatPlugin (line 155)
```rust
let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, false);
```
File: `crates/rspack_plugin_runtime/src/common_js_chunk_format.rs`

## Extension Points

The generated code goes through `render_startup` hook:

```rust
define_hook!(JavascriptModulesRenderStartup: 
  Series(
    compilation: &Compilation, 
    chunk_ukey: &ChunkUkey, 
    module: &ModuleIdentifier, 
    source: &mut RenderSource  // Can be modified!
  )
);
```

### Plugins Using This Hook
- AssignLibraryPlugin - Assign exports to window/object
- ModuleLibraryPlugin - Module.exports compatibility
- ModernModuleLibraryPlugin - Modern format support
- EmbedFederationRuntimePlugin - Module Federation
- ModuleChunkFormatPlugin - Module format

## Example: Injecting Custom Code

```rust
#[plugin_hook(JavascriptModulesRenderStartup for MyPlugin)]
async fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &ModuleIdentifier,
  render_source: &mut RenderSource,  // Mutable!
) -> Result<()> {
  let mut source = ConcatSource::default();
  
  // Add pre-startup code
  source.add(RawStringSource::from("console.log('Before startup');"));
  
  // Add generated startup
  source.add(render_source.source.clone());
  
  // Add post-startup code
  source.add(RawStringSource::from("console.log('After startup');"));
  
  render_source.source = source.boxed();
  Ok(())
}
```

## Key Decision Logic

```
Has dependent chunks?
├─ NO  → Simple: var __webpack_exports__ = (__webpack_exec__(0));
│
└─ YES → Complex with callback
   ├─ passive=true  → Two-phase ON_CHUNKS_LOADED
   └─ passive=false → Single STARTUP_ENTRYPOINT
```

## Helper Functions Used

- `stringify_chunks_to_array(&chunks_ids)` - Converts chunk IDs to JSON array string
- `get_all_chunks()` - Finds all dependent chunks
- `RawStringSource::from()` - Creates source wrapper

## Return Type

```rust
BoxSource  // rspack_sources::BoxSource
           // Type-erased, heap-allocated source implementation
           // Can be ConcatSource, RawStringSource, etc.
```

