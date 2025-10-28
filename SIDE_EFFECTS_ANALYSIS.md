# Side Effects Analysis - mf_async_startup Feature

## Summary
Analysis of potential unintended side effects across all modified files when `mf_async_startup` experiment is **disabled** (false).

## Critical Finding
**CRITICAL ISSUE DETECTED**: The changes introduce UNCONDITIONAL code execution in addition to conditionally-gated code, which could cause regressions when the feature is disabled.

---

## 1. CODE RUNNING WHEN mf_async_startup=false (HIGH PRIORITY)

### 1.1 JavaScript Plugin - Unconditional Behavior Changes
**File**: `crates/rspack_plugin_javascript/src/plugin/mod.rs` (Lines 327-650)

#### Issue: New Variable Declarations (Always Execute)
```rust
// Lines 327-329 - ALWAYS runs, even when mf_async_startup=false
let is_federation_async = compilation.options.experiments.mf_async_startup
  && runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);

// Lines 357-358 - ALWAYS runs  
let mut federation_entry_calls: Vec<String> = Vec::new();
let mut all_chunk_ids: Vec<String> = Vec::new();
```

**Impact**: 
- Extra memory allocation and vector initialization happens unconditionally
- When mf_async_startup=false, these vectors are created but never used
- Loop iterations still process all entries even when not using federation async

#### Issue: New Loop Logic (Always Evaluates)
```rust
// Lines 449-459 - CONDITIONAL blocks but inside UNCONDITIONAL loop processing
if is_federation_async {
  // This path is taken when experiment is true
  federation_entry_calls.push(...);
  for chunk_id in &chunk_ids { ... }
} else if !chunk_ids.is_empty() {
  // Original code path preserved, but wrapped
  buf2.push(...);
}
```

**Issue**: The loop structure changed - variables are collected first, THEN decision logic applied. Previously, decisions were made inline.

#### Issue: New Code Paths for STARTUP_ENTRYPOINT
```rust
// Lines 569-593 - NEW conditional for STARTUP_ENTRYPOINT
if !is_federation_async && runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
  // Async startup entrypoint (for MF async startup)
  allow_inline_startup = false;
  header.push(format!("// the startup function (async)\n{} = {};\n", ...));
  ...
}
```

**Issue**: When mf_async_startup=false, but STARTUP_ENTRYPOINT is present, new code path executes that didn't before. This could affect existing MF flows that don't use this experiment.

#### Issue: New STARTUP_NO_DEFAULT Handling
```rust
// Lines 637-641 - NEW conditional block OUTSIDE entry module check
else if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
  startup.push("// When STARTUP_ENTRYPOINT is present (async MF startup), call .X() instead of .x()");
  startup.push(format!("var __webpack_exports__ = {}();", RuntimeGlobals::STARTUP_ENTRYPOINT).into());
}
```

**Issue**: This code executes when STARTUP_NO_DEFAULT is set AND STARTUP_ENTRYPOINT is present, regardless of mf_async_startup value. This is a new code path.

---

### 1.2 Array Push Callback Chunk Format Plugin
**File**: `crates/rspack_plugin_runtime/src/array_push_callback_chunk_format.rs` (Lines 56-62, 155-156)

#### Issue: Conditional Runtime Requirement Selection
```rust
// Lines 56-62 - NEW conditional logic
if compilation.options.experiments.mf_async_startup {
  runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
} else {
  runtime_requirements.insert(RuntimeGlobals::ON_CHUNKS_LOADED);  // ALWAYS inserts one
}
runtime_requirements.insert(RuntimeGlobals::EXPORTS);
runtime_requirements.insert(RuntimeGlobals::REQUIRE);
```

**Issue**: Previously, always inserted ON_CHUNKS_LOADED. Now when mf_async_startup=false, still inserts ON_CHUNKS_LOADED (correct), BUT the change in behavior to conditionally insert STARTUP_ENTRYPOINT affects downstream requirements.

#### Issue: Passive Flag Inverted
```rust
// Lines 155-156 - BEHAVIOR CHANGE
// OLD: let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, true);
// NEW:
let passive = !compilation.options.experiments.mf_async_startup;
let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, passive);
```

**Impact**: When mf_async_startup=false, passive=true (same as before). But this inverts the semantic meaning - the flag behavior is now tied to experiment flag.

---

### 1.3 Startup Chunk Dependencies Plugin
**File**: `crates/rspack_plugin_runtime/src/startup_chunk_dependencies.rs` (Lines 32-37)

#### Issue: NEW Early Return Condition
```rust
// Lines 32-37 - NEW UNCONDITIONAL check
let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &self.chunk_loading, compilation);

// Skip adding STARTUP if STARTUP_ENTRYPOINT is already present (async MF startup takes precedence)
if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
  return Ok(());  // EARLY EXIT - NEW CODE PATH
}
```

**Issue**: This is a NEW code path that didn't exist before. If any code sets STARTUP_ENTRYPOINT (regardless of mf_async_startup), this plugin now skips adding STARTUP. This could break existing flows where STARTUP_ENTRYPOINT is used for other reasons.

**Regression Risk**: High - affects all chunks that have STARTUP_ENTRYPOINT set, not just federation async startup.

---

## 2. MODIFIED PLUGIN SIGNATURES (Function Behavior Changes)

### 2.1 Enable Chunk Loading Plugin
**File**: `crates/rspack_plugin_runtime/src/lib.rs` (Lines 36-77)

#### Signature Change
```rust
// OLD: pub fn enable_chunk_loading_plugin(loading_type: ChunkLoadingType, plugins: &mut Vec<BoxPlugin>)
// NEW:
pub fn enable_chunk_loading_plugin(loading_type: ChunkLoadingType, mf_async_startup: bool, plugins: &mut Vec<BoxPlugin>)
```

#### JSONP Case - Unconditional Plugin Addition
```rust
// Lines 38-45 - NEW logic for JSONP
ChunkLoadingType::Jsonp => {
  if mf_async_startup {
    plugins.push(
      StartupChunkDependenciesPlugin::new(ChunkLoading::Enable(ChunkLoadingType::Jsonp), true)
        .boxed(),
    );
  }
  plugins.push(JsonpChunkLoadingPlugin::default().boxed());  // ALWAYS added
}
```

**Issue**: JsonpChunkLoadingPlugin is added unconditionally, but StartupChunkDependenciesPlugin is conditional.

#### REQUIRE Case - Behavior Change
```rust
// Lines 47-53 - CHANGED BEHAVIOR
ChunkLoadingType::Require => {
  plugins.push(
    StartupChunkDependenciesPlugin::new(ChunkLoading::Enable(ChunkLoadingType::Require), mf_async_startup)  // CHANGED: was false, now mf_async_startup
      .boxed(),
  );
  plugins.push(CommonJsChunkLoadingPlugin::new(mf_async_startup).boxed())  // CHANGED: was false, now mf_async_startup
}
```

**Critical Issue**: The REQUIRE case ALWAYS had `false` for async_chunk_loading before. Now when mf_async_startup=false, it still gets false (correct), BUT when mf_async_startup=true, it gets true (new behavior). This affects non-MF code that uses REQUIRE chunk loading.

#### Web Worker Hardcoding
**File**: `crates/rspack_plugin_web_worker_template/src/lib.rs` (Line 6)

```rust
// Lines 5-6 - HARDCODED TRUE
enable_chunk_loading_plugin(ChunkLoadingType::ImportScripts, true, plugins);
```

**Issue**: The comment says "ImportScripts always uses async_chunk_loading: true (hardcoded in enable_chunk_loading_plugin)" but it's actually hardcoded in the caller. If someone changes the enable_chunk_loading_plugin default, this breaks.

---

## 3. MODULE FEDERATION PLUGIN CHANGES (Unconditional When Disabled)

### 3.1 Embed Federation Runtime Plugin
**File**: `crates/rspack_plugin_mf/src/container/embed_federation_runtime_plugin.rs`

#### Issue: STARTUP vs STARTUP_ENTRYPOINT Logic
```rust
// Lines 80-87 - Adds STARTUP or STARTUP_ENTRYPOINT based on mf_async_startup
if is_enabled {
  if compilation.options.experiments.mf_async_startup {
    runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
  } else {
    runtime_requirements.insert(RuntimeGlobals::STARTUP);
  }
}
```

**Issue**: This ALWAYS inserts one of them when federation is enabled. When mf_async_startup=false, inserts STARTUP (original behavior). But this is independent of the js_plugin's startup logic, creating duplicate requirement insertions.

#### Issue: NEW Async Startup Pattern in render_startup
```rust
// Lines 207-227 - NEW CODE PATH when mf_async_startup=true
if compilation.options.experiments.mf_async_startup {
  // Use federation async startup pattern with Promise.all wrapping
  // This completely replaces the original startup since Promise.all handles everything
  startup_with_call.add(RawStringSource::from("\n// Federation async startup (delegated)\n"));
  startup_with_call.add(RawStringSource::from("var promises = [];\n"));
  ...
  render_source.source = startup_with_call.boxed();  // REPLACES original
}
```

**Issue**: When mf_async_startup=false, the else branch handles it:
```rust
} else {
  // Standard sync startup call - prepend to original
  startup_with_call.add(RawStringSource::from("\n// Federation startup call\n"));
  startup_with_call.add(RawStringSource::from(format!("{}();\n", RuntimeGlobals::STARTUP.name())));
  startup_with_call.add(render_source.source.clone());  // PREPENDS original
}
```

**Potential Issue**: The async case REPLACES source, sync case PREPENDS. This is a behavioral difference that could cause issues if the original source is modified elsewhere.

---

### 3.2 Embed Federation Runtime Module
**File**: `crates/rspack_plugin_mf/src/container/embed_federation_runtime_module.rs`

#### Issue: Startup Name Selection
```rust
// Lines 92-96 - NEW logic
let startup = if compilation.options.experiments.mf_async_startup {
  RuntimeGlobals::STARTUP_ENTRYPOINT.name()
} else {
  RuntimeGlobals::STARTUP.name()
};
```

**Impact**: The prevStartup wrapper pattern is generated differently based on which startup function is targeted. When mf_async_startup=false, targets STARTUP (original). But this is conditional behavior that didn't exist before.

---

### 3.3 Module Federation Runtime Plugin
**File**: `crates/rspack_plugin_mf/src/container/module_federation_runtime_plugin.rs`

#### Issue: Conditional STARTUP_ENTRYPOINT Requirements
```rust
// Lines 49-63 - NEW CONDITIONAL requirements insertion
if compilation.options.experiments.mf_async_startup {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  if chunk.has_runtime(&compilation.chunk_group_by_ukey)
    && compilation.chunk_graph.get_number_of_entry_modules(chunk_ukey) > 0
  {
    runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
  }
}
```

**Issue**: When mf_async_startup=false, this block is skipped entirely, so STARTUP_ENTRYPOINT is NOT inserted here. But it might be inserted elsewhere (by embed_federation_runtime_plugin), creating inconsistency.

---

## 4. CONFIG/TYPE CHANGES

### 4.1 Type System Changes
**Files**: 
- `packages/rspack/src/config/types.ts` (modified)
- `packages/rspack/src/config/normalization.ts` (modified)
- `crates/rspack_binding_api/src/raw_options/raw_experiments/mod.rs` (modified)
- `crates/rspack_binding_api/src/lib.rs` (modified)

#### Issue: New Parameter Threading
The `mf_async_startup` parameter now flows through:
1. Raw options parsing
2. Config normalization
3. Raw builtins module

```rust
// crates/rspack_binding_api/src/raw_options/raw_builtins/mod.rs
// Function signature changed:
pub fn apply(
  &self,
  env: Env,
  compiler_object: &mut Object,
  mf_async_startup: bool,  // NEW PARAMETER
  plugins: &mut Vec<BoxPlugin>,
) -> napi::Result<()>
```

**Impact**: The parameter must be passed through all call sites. Any missing call site could cause wrong behavior (false default).

**Call Sites**:
```rust
// crates/rspack_binding_api/src/raw_options/raw_builtins/mod.rs - Line 393
enable_chunk_loading_plugin(chunk_loading_type.as_str().into(), mf_async_startup, plugins);

// crates/rspack/src/builder/builder_context.rs - Line 146
rspack_plugin_runtime::enable_chunk_loading_plugin(
  chunk_loading_type, 
  compiler_options.experiments.mf_async_startup,  // ALWAYS PASSED
  &mut plugins
);
```

**Verification**: Both call sites correctly pass the parameter.

---

## 5. REGRESSION RISK ASSESSMENT

### Critical Regressions (High Probability)
1. **STARTUP_ENTRYPOINT handling in non-federation code**: Lines 637-641 of js_plugin/mod.rs now execute for ANY code that has STARTUP_ENTRYPOINT set, not just federation async startup.

2. **Startup chunk dependencies early exit**: The new early return at line 35 of startup_chunk_dependencies.rs will skip STARTUP insertion for ANY chunk with STARTUP_ENTRYPOINT, not just federation async startup.

3. **Array push callback passive flag inversion**: The inverted semantic (passive = !mf_async_startup) could affect entry startup generation logic unexpectedly.

### Moderate Regressions (Medium Probability)
4. **Enable chunk loading plugin signature change**: REQUIRE case now conditionally passes mf_async_startup instead of hardcoded false.

5. **Duplicate runtime requirements insertion**: Both embed_federation_runtime_plugin and js_plugin may insert STARTUP/STARTUP_ENTRYPOINT.

6. **Unconditional variable allocation**: Extra vectors created in js_plugin loop even when not used.

### Low Regressions (Low Probability)
7. **Web worker hardcoding**: Explicit true passed instead of implicit default.

8. **Module federation plugin order dependencies**: Multiple plugins affecting same requirements.

---

## 6. CODE NOT PROTECTED BY mf_async_startup FLAG

### Variables Always Allocated (js_plugin/mod.rs)
```
- is_federation_async (line 328)
- federation_entry_calls (line 357)
- all_chunk_ids (line 358)
```

### Always-Executed Logic Changes
- Loop processing changes (lines 449-459)
- STARTUP_ENTRYPOINT checks (lines 569-593, 637-641)
- STARTUP_NO_DEFAULT handling (lines 637-641)

### Plugin Signature Changes
- `enable_chunk_loading_plugin` now requires mf_async_startup parameter
- All callers must pass it (no default, breaking change risk)

---

## 7. UNTESTED EDGE CASES

### When mf_async_startup=false but STARTUP_ENTRYPOINT is set:
- Which code path executes in js_plugin lines 569-593?
- Does startup_chunk_dependencies early exit interfere?
- Are there duplicate runtime requirements?

### When both STARTUP and STARTUP_ENTRYPOINT are required:
- What's the loading/execution order?
- Can promises and sync code execute together?

### Commonwealth Require Chunk Loading Case:
- Changed from hardcoded false to mf_async_startup value
- How does this affect non-MF code?

---

## Recommendations

1. **Add explicit guards**: Change lines that check `!is_federation_async` to also verify `mf_async_startup` is actually true, not just STARTUP_ENTRYPOINT is present.

2. **Extract variable allocation**: Move federation_entry_calls and all_chunk_ids inside the `if is_federation_async` block to avoid unconditional allocation.

3. **Add regression tests**: Test these specific scenarios:
   - mf_async_startup=false with STARTUP_ENTRYPOINT already set
   - Non-federation code paths that use STARTUP_ENTRYPOINT
   - REQUIRE chunk loading with various option combinations

4. **Document behavior changes**: The inverted passive flag and new code paths should be documented as breaking changes for non-federation use cases.

5. **Consider feature flag boundaries**: The changes to startup_chunk_dependencies and embed_federation_runtime_plugin affect non-federation code, which violates the principle of minimal scope.

