# Specific Code Issues - Side Effects Analysis

## Issue #1: CRITICAL - STARTUP_ENTRYPOINT Handling in Non-Federation Code

### Location
File: `crates/rspack_plugin_javascript/src/plugin/mod.rs`
Lines: 637-641 (outer else if) + 569-593 (first else if)

### The Problem
This code executes for ANY chunk that has STARTUP_ENTRYPOINT set, not just federation async startup:

```rust
    } else if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
      // When STARTUP_ENTRYPOINT is present (async MF startup), call .X() instead of .x()
      startup.push("// run startup".into());
      startup.push(format!("var __webpack_exports__ = {}();", RuntimeGlobals::STARTUP_ENTRYPOINT).into());
    }
```

**Current Issue**: The check is `if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT)` - this doesn't verify that mf_async_startup is enabled. Any code path that sets STARTUP_ENTRYPOINT (federation or not) will execute this.

**Also**: Lines 569-593 have similar issue:
```rust
if !is_federation_async && runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
  // This wraps startup functions
  allow_inline_startup = false;
  header.push(format!("// the startup function (async)\n{} = {};\n", ...));
}
```

**Why it's bad**: 
- `is_federation_async` is false when mf_async_startup=false
- But STARTUP_ENTRYPOINT might be set by other code
- This creates a "ghost" startup function wrapping

### Fix Required
```rust
// ADD THIS CHECK:
if !is_federation_async && mf_async_startup && runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
  // ... only then execute this new code
}

// OR BETTER:
if is_federation_async && runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
  // Original federation async path only
}
```

---

## Issue #2: CRITICAL - Unconditional Early Exit in Startup Dependencies

### Location
File: `crates/rspack_plugin_runtime/src/startup_chunk_dependencies.rs`
Lines: 32-37

### The Problem
```rust
let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &self.chunk_loading, compilation);

// Skip adding STARTUP if STARTUP_ENTRYPOINT is already present (async MF startup takes precedence)
if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
  return Ok(());  // <-- EARLY EXIT - NEW CODE
}

if compilation
  .chunk_graph
  .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey)
  && is_enabled_for_chunk
{
  runtime_requirements.insert(RuntimeGlobals::STARTUP);
  // ...
}
```

**The Issue**: This early return executes for ANY code that has STARTUP_ENTRYPOINT set, regardless of mf_async_startup. This means:
- If your federation code uses STARTUP_ENTRYPOINT (legitimately, not async)
- The STARTUP requirement never gets added
- Chunk dependencies won't load properly

**Why it's bad**:
- Assumes STARTUP_ENTRYPOINT means "async MF startup"
- But STARTUP_ENTRYPOINT could be set for other async startup patterns
- Breaks existing Module Federation code that uses STARTUP_ENTRYPOINT but not async startup

### Fix Required
```rust
// OPTION 1: Only skip if actually async federation
if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) 
   && compilation.options.experiments.mf_async_startup {
  return Ok(());  // Only skip if async federation is enabled
}

// OPTION 2: More specific - check for federation runtime markers
if is_federation_async_startup_detected {
  return Ok(());
}
```

---

## Issue #3: CRITICAL - REQUIRE Chunk Loading Behavior Change

### Location
File: `crates/rspack_plugin_runtime/src/lib.rs`
Lines: 47-53

### The Problem
```rust
ChunkLoadingType::Require => {
  plugins.push(
    StartupChunkDependenciesPlugin::new(
      ChunkLoading::Enable(ChunkLoadingType::Require), 
      mf_async_startup  // <-- CHANGED FROM: false
    )
    .boxed(),
  );
  plugins.push(CommonJsChunkLoadingPlugin::new(mf_async_startup).boxed())  // <-- CHANGED FROM: false
}
```

**Before Changes:**
```rust
ChunkLoadingType::Require => {
  plugins.push(
    StartupChunkDependenciesPlugin::new(ChunkLoading::Enable(ChunkLoadingType::Require), false)
      .boxed(),
  );
  plugins.push(CommonJsChunkLoadingPlugin::new(false).boxed())
}
```

**The Issue**: 
- REQUIRE chunk loading is not federation-specific
- But now when mf_async_startup=true, it behaves differently
- This affects ALL require()-based chunk loading, not just federation

**Impact Scenarios**:
1. Non-federation Node.js app uses REQUIRE chunk loading
2. App has mf_async_startup=true for federation in one entry
3. All other entries now get async chunk loading behavior
4. Breaking change for synchronous code

**Why it's bad**:
- Feature coupling: federation feature affects non-federation code
- Breaking change: existing code expects synchronous require behavior
- Silent: No error, just different async behavior

### Fix Required
```rust
ChunkLoadingType::Require => {
  // Only pass mf_async_startup if this is for federation
  // Check if this is being called for federation context
  let async_for_federation = mf_async_startup; // keep as-is for now
  
  // OR: Only enable async for federation entry chunks
  // Requires more context about which chunks need this
  plugins.push(
    StartupChunkDependenciesPlugin::new(
      ChunkLoading::Enable(ChunkLoadingType::Require), 
      false  // REVERT: Keep synchronous for non-federation require
    )
    .boxed(),
  );
}
```

---

## Issue #4: HIGH - Unconditional Variable Allocation

### Location
File: `crates/rspack_plugin_javascript/src/plugin/mod.rs`
Lines: 357-358

### The Problem
```rust
// For federation async startup, collect entry data
let mut federation_entry_calls: Vec<String> = Vec::new();  // <-- ALLOCATED ALWAYS
let mut all_chunk_ids: Vec<String> = Vec::new();           // <-- ALLOCATED ALWAYS

for (i, (module, entry)) in entries.iter().enumerate() {
  // ... 100+ lines of loop code
  
  if is_federation_async {
    federation_entry_calls.push(format!("__webpack_exec__({})", module_id_expr));
    for chunk_id in &chunk_ids {
      if !all_chunk_ids.contains(chunk_id) {
        all_chunk_ids.push(chunk_id.clone());
      }
    }
  } else if !chunk_ids.is_empty() {
    // ... original code
  }
}
```

**The Issue**:
- Vectors created unconditionally
- Only used when `is_federation_async=true`
- When `is_federation_async=false`, vectors stay empty
- Wastes memory for every entry module

**Why it's bad**:
- Performance impact: Vector allocation for every build
- Memory waste: Empty vectors that serve no purpose
- Code smell: Variables exist in wrong scope

### Fix Required
```rust
// Move inside the federation async branch:
if is_federation_async {
  let mut federation_entry_calls: Vec<String> = Vec::new();
  let mut all_chunk_ids: Vec<String> = Vec::new();
  
  for (i, (module, entry)) in entries.iter().enumerate() {
    // ... collection code
    federation_entry_calls.push(...);
    all_chunk_ids.push(...);
  }
  
  // Use federation_entry_calls and all_chunk_ids here
} else {
  // Original code
}
```

---

## Issue #5: MEDIUM - Passive Flag Semantic Inversion

### Location
File: `crates/rspack_plugin_runtime/src/array_push_callback_chunk_format.rs`
Lines: 155-156

### The Problem
```rust
// OLD:
let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, true);

// NEW:
let passive = !compilation.options.experiments.mf_async_startup;
let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, passive);
```

**The Issue**:
- Before: Always passed `true` (passive)
- Now: Passes `!mf_async_startup`
- When mf_async_startup=false: passive=true (same as before) ✓
- When mf_async_startup=true: passive=false (new behavior)

**Why it's concerning**:
- Semantic change to the flag
- The flag behavior is now tied to mf_async_startup
- If someone debugs this, they see confusing behavior

**Example of confusion**:
```
Developer: "Why is passive false?"
Code says: "because !true = false"
But really: "because mf_async_startup=true"
```

### Fix Required
```rust
// More explicit:
let passive = if compilation.options.experiments.mf_async_startup {
  false  // For async MF startup, NOT passive
} else {
  true   // For normal startup, passive mode
};

// OR with comments:
// Passive mode is disabled for federation async startup (which needs active control)
let passive = !compilation.options.experiments.mf_async_startup;
```

---

## Issue #6: MEDIUM - Duplicate Runtime Requirements

### Location
File: `crates/rspack_plugin_mf/src/container/embed_federation_runtime_plugin.rs`
Lines: 80-87

### The Problem
```rust
// In EmbedFederationRuntimePlugin:
if is_enabled {
  if compilation.options.experiments.mf_async_startup {
    runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
  } else {
    runtime_requirements.insert(RuntimeGlobals::STARTUP);
  }
}

// ALSO in JsPlugin (rspack_plugin_javascript/mod.rs):
if is_federation_async {
  // ... NEW code path inserts STARTUP_ENTRYPOINT
} else if !is_federation_async && runtime_requirements.contains(RuntimeGlobals::STARTUP) {
  // ... inserts STARTUP
}
```

**The Issue**:
- Both plugins insert startup requirements
- Could insert the same requirement twice
- Unclear execution order
- Runtime requirements are supposed to be sets (no duplicates) but the logic is fragile

**Why it's bad**:
- Multiple code paths affecting same runtime requirements
- Hard to debug which plugin is responsible
- Potential for conflicts if plugins have different assumptions

### Investigation Needed
```rust
// Check if this causes issues:
if (is_federation_async) {
  // Does embed_federation_runtime_plugin also insert?
  // Check: embed_federation_runtime_plugin.rs lines 80-87
  // YES - it inserts STARTUP_ENTRYPOINT for mf_async_startup=true
  
  // So STARTUP_ENTRYPOINT could be inserted TWICE
  // But Sets prevent duplicates, so probably OK
  // Still bad code smell though
}
```

---

## Issue #7: LOW - Web Worker Hardcoding Comment/Reality Mismatch

### Location
File: `crates/rspack_plugin_web_worker_template/src/lib.rs`
Lines: 5-6

### The Problem
```rust
// Comment says: "ImportScripts always uses async_chunk_loading: true (hardcoded in enable_chunk_loading_plugin)"
// But code is:
enable_chunk_loading_plugin(ChunkLoadingType::ImportScripts, true, plugins);
//                                                            ^^^^  
//                                    HARDCODED HERE, not in function

// The actual function:
pub fn enable_chunk_loading_plugin(
  loading_type: ChunkLoadingType, 
  mf_async_startup: bool,
  plugins: &mut Vec<BoxPlugin>
) {
  match loading_type {
    ChunkLoadingType::ImportScripts => {
      plugins.push(
        StartupChunkDependenciesPlugin::new(
          ChunkLoading::Enable(ChunkLoadingType::ImportScripts),
          true,  // <-- HERE is where it's truly hardcoded
        )
        .boxed(),
      );
      // ...
    }
  }
}
```

**The Issue**:
- Comment says it's hardcoded in function
- It's actually hardcoded in the caller
- Misleading documentation

**Why it's bad**:
- Maintenance confusion
- If someone refactors the function, they might break this without realizing
- Comment doesn't match implementation

### Fix Required
```rust
pub fn web_worker_template_plugin(plugins: &mut Vec<BoxPlugin>) {
  plugins.push(ArrayPushCallbackChunkFormatPlugin::default().boxed());
  // Note: ImportScripts always uses async chunk loading (true is hardcoded here)
  // This is intentional for web workers which always support async loading
  enable_chunk_loading_plugin(ChunkLoadingType::ImportScripts, true, plugins);
}
```

---

## Summary Table of Issues

| Issue | Severity | Type | Affects | Risk |
|-------|----------|------|---------|------|
| #1 | CRITICAL | Code path | Non-federation STARTUP_ENTRYPOINT | HIGH |
| #2 | CRITICAL | Early exit | Any STARTUP_ENTRYPOINT user | HIGH |
| #3 | CRITICAL | Behavior change | REQUIRE chunk loading | CRITICAL |
| #4 | HIGH | Performance | All entry modules | MEDIUM |
| #5 | MEDIUM | Semantic | Array push entry startup | MEDIUM |
| #6 | MEDIUM | Duplication | Federation plugins | MEDIUM |
| #7 | LOW | Documentation | Web worker template | LOW |

---

## Test Cases Needed

```
Test #1: Non-federation code with STARTUP_ENTRYPOINT
- Config: { mfAsyncStartup: false, entry: {...} }
- Expected: STARTUP_ENTRYPOINT wrapping should NOT occur
- Currently: LIKELY FAILS (issue #1)

Test #2: Federation + REQUIRE chunk loading
- Config: { mfAsyncStartup: true, experimentChunkLoading: 'require' }
- Expected: Normal sync require() calls
- Currently: MIGHT USE ASYNC (issue #3)

Test #3: Array push callback with async disabled
- Config: { mfAsyncStartup: false, chunkFormat: 'array-push' }
- Expected: Passive mode entry startup generation
- Currently: UNCERTAIN (issue #5)

Test #4: Startup chunk dependencies with STARTUP_ENTRYPOINT
- Config: Non-federation, STARTUP_ENTRYPOINT set externally
- Expected: STARTUP should still be added
- Currently: FAILS (issue #2)
```

