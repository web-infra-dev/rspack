# Index: generate_entry_startup Function Research

## Quick Navigation

This research contains comprehensive analysis of the `generate_entry_startup` function in rspack's runtime plugin system.

### Generated Documents

1. **GENERATE_ENTRY_STARTUP_RESEARCH.md** (23 KB)
   - Primary comprehensive document
   - 7 detailed sections
   - ~5000 words
   - Covers all aspects from parameters to extension points
   - Best for: Complete understanding

2. **GENERATE_ENTRY_STARTUP_QUICK_REF.md** (5 KB)
   - One-page quick reference
   - Condensed key information
   - Decision trees and flowcharts
   - Best for: Desk reference while coding

3. **GENERATE_ENTRY_STARTUP_CODE_SNIPPETS.md** (14 KB)
   - Complete code implementations
   - 8 sections with actual source code
   - Real examples from the codebase
   - Generated output examples
   - Best for: Implementation details

4. **GENERATE_ENTRY_STARTUP_SUMMARY.txt** (9 KB)
   - Executive summary
   - Key findings
   - Document guide
   - Questions answered checklist
   - Best for: Overview and orientation

---

## Document Structure

### RESEARCH.md Sections

```
1. Overview
2. Function Signature and Parameters (4 tables)
3. The `passive` Parameter - Impact (2 modes detailed)
4. Runtime Globals Used (4 globals documented)
5. Where This Function Is Called From (2 call sites)
6. Structure of Generated Code (4 phases)
7. Extension and Injection Points (hook system)
8. Key Code Sections - Detailed Breakdown
```

### QUICK_REF.md Sections

```
- File Location & Line Numbers
- Function Signature
- Parameters Quick Reference
- passive Parameter Effect
- Code Generation Phases (4 phases)
- Runtime Globals Table
- Where It's Called (2 locations)
- Extension Points & Hooks
- Example: Injecting Custom Code
- Key Decision Logic
- Helper Functions
- Return Type
```

### CODE_SNIPPETS.md Sections

```
1. Complete Function (helpers.rs:181-273)
2. Call Site 1: ArrayPushCallbackChunkFormatPlugin
3. Call Site 2: CommonJsChunkFormatPlugin
4. Hook Definition (drive.rs)
5. Example Implementation: AssignLibraryPlugin
6. Helper: stringify_chunks_to_array
7. Runtime Globals Definition
8. Generated Output Examples (4 examples)
```

---

## Quick Answers

### Q: What does generate_entry_startup do?
Generate JavaScript code that executes entry modules, handling:
- Entry module identification
- Chunk dependency resolution
- Runtime global selection
- Module wrapper creation

**See**: RESEARCH.md Section 1, CODE_SNIPPETS.md Complete Function

### Q: What are the parameters?
```rust
pub fn generate_entry_startup(
  compilation: &Compilation,       // Module/chunk metadata
  chunk: &ChunkUkey,               // Current chunk key
  entries: &IdentifierLinkedMap,   // Entry module map
  passive: bool,                   // ON_CHUNKS_LOADED vs STARTUP_ENTRYPOINT
) -> BoxSource
```

**See**: RESEARCH.md Section 1, QUICK_REF.md Parameters Table

### Q: What does passive parameter do?
- **passive=true**: Uses `__webpack_require__.O` (two-phase deferred)
- **passive=false**: Uses `__webpack_require__.X` (single async call)

**See**: RESEARCH.md Section 2, QUICK_REF.md passive Parameter Effect

### Q: What runtime globals are used?
- **ENTRY_MODULE_ID** (.s): Stores the entry module ID
- **ON_CHUNKS_LOADED** (.O): Deferred callback queue (passive mode)
- **STARTUP_ENTRYPOINT** (.X): Async executor (active mode)

**See**: RESEARCH.md Section 3, QUICK_REF.md Runtime Globals Table

### Q: Where is this function called?
1. **ArrayPushCallbackChunkFormatPlugin** (array_push_callback_chunk_format.rs:156)
   - Determines passive based on `!mf_async_startup` flag

2. **CommonJsChunkFormatPlugin** (common_js_chunk_format.rs:155)
   - Always passes passive=false

**See**: RESEARCH.md Section 4, CODE_SNIPPETS.md Call Sites 1&2

### Q: How can I inject custom code?
Via the `JavascriptModulesRenderStartup` hook:

1. Tap the hook in your plugin
2. Clone the render_source
3. Wrap or modify the source
4. Replace render_source.source

**See**: RESEARCH.md Section 6, QUICK_REF.md Example, CODE_SNIPPETS.md AssignLibraryPlugin

### Q: What does the generated code look like?

Simple (no dependencies):
```javascript
var __webpack_exec__ = function(moduleId) { 
  return __webpack_require__(__webpack_require__.s = moduleId) 
}
var __webpack_exports__ = (__webpack_exec__(0));
```

Complex with chunks (passive=true):
```javascript
__webpack_require__.O(0, [1,2,3], function() {
  return __webpack_exec__(0);
});
var __webpack_exports__ = __webpack_require__.O();
```

Complex with chunks (passive=false):
```javascript
var __webpack_exports__ = __webpack_require__.X(0, [1,2,3], function() {
  return __webpack_exec__(0);
});
```

**See**: CODE_SNIPPETS.md Generated Output Examples

---

## Reading Order

### For Complete Understanding
1. Start with GENERATE_ENTRY_STARTUP_SUMMARY.txt (2 min read)
2. Read GENERATE_ENTRY_STARTUP_RESEARCH.md sections 1-3 (10 min)
3. Review CODE_SNIPPETS.md "Complete Function" (5 min)
4. Read RESEARCH.md sections 4-7 (15 min)
5. Reference QUICK_REF.md as needed

**Total**: ~45 minutes for comprehensive understanding

### For Quick Reference
1. Check GENERATE_ENTRY_STARTUP_QUICK_REF.md
2. Use Ctrl+F to find what you need
3. Reference specific line numbers

**Total**: 5-10 minutes per lookup

### For Implementation
1. Open CODE_SNIPPETS.md to the relevant section
2. Copy the pattern you need
3. Refer to RESEARCH.md Section 6 for hook details
4. Use QUICK_REF.md template for custom code

**Total**: 15-20 minutes for implementation

---

## Key Concepts

### Startup Execution Modes

**Mode 1: Simple (No Chunks)**
- Single line execution
- No dependency resolution needed
- Most efficient

**Mode 2: Complex with ON_CHUNKS_LOADED (passive=true)**
- Two-phase execution (register + execute)
- Used in array-push-callback format
- When mf_async_startup=false

**Mode 3: Complex with STARTUP_ENTRYPOINT (passive=false)**
- Single call with async callback
- Used in CommonJS format
- Can return Promise if async template used

### The __webpack_exec__ Wrapper

```javascript
var __webpack_exec__ = function(moduleId) { 
  return __webpack_require__(__webpack_require__.s = moduleId) 
}
```

This wrapper:
1. Sets the global entry module ID (__webpack_require__.s)
2. Requires the module with that ID
3. Returns the module's exports

This ensures the runtime knows which module is the entry point.

### Hook System for Extension

The `JavascriptModulesRenderStartup` hook allows:
- **Library plugins**: Assign exports to window/module.exports
- **Custom plugins**: Inject telemetry, logging, custom code
- **Framework plugins**: Add framework-specific initialization

All modifications happen after code generation but before final output.

---

## File Locations Reference

| File | Lines | Purpose |
|------|-------|---------|
| helpers.rs | 181-273 | Main function |
| array_push_callback_chunk_format.rs | 156 | Call site 1 |
| common_js_chunk_format.rs | 155 | Call site 2 |
| drive.rs | 13, 32 | Hook definition |
| assign_library_plugin.rs | 245 | Hook implementation example |
| runtime.rs | 370 | stringify_chunks_to_array |
| runtime_globals.rs | 86, 185, 195 | Runtime globals |

---

## Important Notes

1. **passive parameter is key**: It controls which runtime method is generated
2. **Module ID extraction filters JavaScript**: Non-JS modules are skipped
3. **Chunk dependencies are collected**: To ensure proper loading order
4. **Hook system is extensible**: Multiple plugins can modify startup code
5. **Two-phase execution (ON_CHUNKS_LOADED)**: Register callback first, then execute queue
6. **Async execution (STARTUP_ENTRYPOINT)**: Single call that can defer via callback

---

## Related Research Documents

These documents provide additional context:
- `ANALYSIS_SUMMARY.md`: Overall rspack startup system
- `STARTUP_BOOTSTRAP_ANALYSIS.md`: Complete startup flow
- `README_STARTUP_ANALYSIS.md`: Detailed startup analysis

---

## Questions or Issues?

If you need more information:
1. Check the relevant section in RESEARCH.md
2. Look at CODE_SNIPPETS.md for implementation details
3. Reference QUICK_REF.md for specific parameters
4. Check the original source files at the line numbers provided

**All files are cross-referenced with line numbers for easy lookup.**

