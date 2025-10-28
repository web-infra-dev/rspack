# Rspack Startup Bootstrap - Code Snippets Reference

## Quick Reference: The Three Startup Paths

### Path Selection Logic (mod.rs:531-545)

```rust
// This is the FINAL decision point that determines which startup path
else if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
  // When STARTUP_ENTRYPOINT is present (async MF startup), call .X() instead of .x()
  startup.push("// run startup".into());
  startup.push(format!("var __webpack_exports__ = {}();", RuntimeGlobals::STARTUP_ENTRYPOINT).into());
} else if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
  header.push(
    format!(
      "// the startup function\n// It's empty as some runtime module handles the default behavior\n{} = function(){{}};",
      RuntimeGlobals::STARTUP
    )
    .into(),
  );
  startup.push("// run startup".into());
  startup.push(format!("var __webpack_exports__ = {}();", RuntimeGlobals::STARTUP).into());
}
```

### Path A1a: STARTUP Wrapper (mod.rs:495-517)

When entry modules exist and STARTUP is required:

```rust
if runtime_requirements.contains(RuntimeGlobals::STARTUP) {
  allow_inline_startup = false;
  header.push(
    format!(
      "// the startup function\n{} = {};\n",
      RuntimeGlobals::STARTUP,
      basic_function(
        &compilation.options.output.environment,
        "",
        &format!("{}\nreturn {}", buf2.join("\n"), RuntimeGlobals::EXPORTS)
      )
    )
    .into(),
  );
  startup.push("// run startup".into());
  startup.push(
    format!(
      "var {} = {}();",
      RuntimeGlobals::EXPORTS,
      RuntimeGlobals::STARTUP
    )
    .into(),
  );
} else {
  // Path A1b: Inline
  startup.push("// startup".into());
  startup.push(buf2.join("\n").into());
}
```

Generated output:
```javascript
// HEADER
__webpack_require__.x = function() {
    var __webpack_exports__ = {};
    // <buf2 code>
    return __webpack_exports__;
};

// STARTUP
// run startup
var __webpack_exports__ = __webpack_require__.x();
```

### Module Execution - Case 1: Dependent Chunks (mod.rs:421-435)

```rust
if !chunk_ids.is_empty() {
  buf2.push(
    format!(
      "{}{}(undefined, {}, function() {{ return {}({module_id_expr}) }});",
      if i + 1 == entries.len() {
        format!("var {} = ", RuntimeGlobals::EXPORTS)
      } else {
        "".to_string()
      },
      RuntimeGlobals::ON_CHUNKS_LOADED,
      stringify_array(&chunk_ids),
      RuntimeGlobals::REQUIRE
    )
    .into(),
  );
}
```

Generated output:
```javascript
var __webpack_exports__ = __webpack_require__.O(undefined, [1, 2], function() { 
  return __webpack_require__(0);
});
```

### Module Execution - Case 2: Use Require (mod.rs:436-448)

```rust
else if use_require {
  buf2.push(
    format!(
      "{}{}({module_id_expr});",
      if i + 1 == entries.len() {
        format!("var {} = ", RuntimeGlobals::EXPORTS)
      } else {
        "".to_string()
      },
      RuntimeGlobals::REQUIRE
    )
    .into(),
  )
}
```

Generated output:
```javascript
var __webpack_exports__ = __webpack_require__(0);
```

### Module Execution - Case 3: Direct Execution (mod.rs:449-483)

```rust
else {
  let should_exec = i + 1 == entries.len();
  if should_exec {
    buf2.push(format!("var {} = {{}}", RuntimeGlobals::EXPORTS).into());
  }
  if require_scope_used {
    buf2.push(
      format!(
        "__webpack_modules__[{module_id_expr}](0, {}, {});",
        if should_exec {
          RuntimeGlobals::EXPORTS.name()
        } else {
          "{}"
        },
        RuntimeGlobals::REQUIRE
      )
      .into(),
    );
  } else if let Some(entry_runtime_requirements) = entry_runtime_requirements
    && entry_runtime_requirements.contains(RuntimeGlobals::EXPORTS)
  {
    buf2.push(
      format!(
        "__webpack_modules__[{module_id_expr}](0, {});",
        if should_exec {
          RuntimeGlobals::EXPORTS.name()
        } else {
          "{}"
        }
      )
      .into(),
    );
  } else {
    buf2.push(format!("__webpack_modules__[{module_id_expr}]();").into());
  }
}
```

Generated outputs:
```javascript
// With require_scope_used
var __webpack_exports__ = {};
__webpack_modules__[0](0, __webpack_exports__, __webpack_require__);

// With exports but not require_scope
var __webpack_exports__ = {};
__webpack_modules__[0](0, __webpack_exports__);

// Neither
__webpack_modules__[0]();
```

## Runtime Requirement Assignment

### STARTUP_ENTRYPOINT vs ON_CHUNKS_LOADED (array_push_callback_chunk_format.rs:56-60)

```rust
if compilation
  .chunk_graph
  .get_number_of_entry_modules(chunk_ukey)
  > 0
{
  if compilation.options.experiments.mf_async_startup {
    // Async Module Federation path
    runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
  } else {
    // Deferred/passive path
    runtime_requirements.insert(RuntimeGlobals::ON_CHUNKS_LOADED);
  }
  runtime_requirements.insert(RuntimeGlobals::EXPORTS);
  runtime_requirements.insert(RuntimeGlobals::REQUIRE);
}
```

### STARTUP Flag Assignment (startup_chunk_dependencies.rs:25-47)

```rust
#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for StartupChunkDependenciesPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &self.chunk_loading, compilation);
  if compilation
    .chunk_graph
    .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey)
    && is_enabled_for_chunk
  {
    // Entry modules have dependent chunks - wrap in startup function
    runtime_requirements.insert(RuntimeGlobals::STARTUP);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
  }
  Ok(())
}
```

## Helper: generate_entry_startup (helpers.rs:181-273)

```rust
pub fn generate_entry_startup(
  compilation: &Compilation,
  chunk: &ChunkUkey,
  entries: &IdentifierLinkedMap<ChunkGroupUkey>,
  passive: bool,  // KEY: Controls behavior
) -> BoxSource {
  let mut module_id_exprs = vec![];
  // ... collect module IDs and dependent chunks ...
  
  let mut source = String::default();
  source.push_str(&format!(
    "var __webpack_exec__ = function(moduleId) {{ return __webpack_require__({} = moduleId) }}\n",
    RuntimeGlobals::ENTRY_MODULE_ID
  ));

  let module_ids_code = &module_id_exprs
    .iter()
    .map(|module_id_expr| format!("__webpack_exec__({module_id_expr})"))
    .collect::<Vec<_>>()
    .join(", ");
    
  if chunks_ids.is_empty() {
    // No dependent chunks - direct execution
    if !module_ids_code.is_empty() {
      source.push_str("var __webpack_exports__ = (");
      source.push_str(module_ids_code);
      source.push_str(");\n");
    }
  } else {
    // Has dependent chunks
    if !passive {
      source.push_str("var __webpack_exports__ = ");
    }
    source.push_str(&format!(
      "{}(0, {}, function() {{\n  return {};\n}});\n",
      if passive {
        RuntimeGlobals::ON_CHUNKS_LOADED
      } else {
        RuntimeGlobals::STARTUP_ENTRYPOINT
      },
      stringify_chunks_to_array(&chunks_ids),
      module_ids_code
    ));
    if passive {
      source.push_str(&format!(
        "var __webpack_exports__ = {}();\n",
        RuntimeGlobals::ON_CHUNKS_LOADED
      ));
    }
  }

  RawStringSource::from(source).boxed()
}
```

### Key Logic: passive flag determination

When called from array_push_callback_chunk_format.rs:156:
```rust
let passive = !compilation.options.experiments.mf_async_startup;
let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, passive);
```

- If `mf_async_startup=true`: `passive=false` → Uses STARTUP_ENTRYPOINT (.X)
- If `mf_async_startup=false`: `passive=true` → Uses ON_CHUNKS_LOADED (.O)

## Runtime Global Definitions (runtime_globals.rs)

```rust
const STARTUP_ENTRYPOINT = 1 << 34;  // __webpack_require__.X
const STARTUP = 1 << 42;              // __webpack_require__.x
const ON_CHUNKS_LOADED = 1 << 15;    // __webpack_require__.O
const EXPORTS = 1 << 44;              // __webpack_exports__
const STARTUP_NO_DEFAULT = 1 << 40;  // Marker flag
```

Naming map:
```rust
impl RuntimeGlobals {
  pub const fn name(&self) -> &'static str {
    match *self {
      R::STARTUP_ENTRYPOINT => "__webpack_require__.X",
      R::STARTUP => "__webpack_require__.x",
      R::ON_CHUNKS_LOADED => "__webpack_require__.O",
      R::EXPORTS => "__webpack_exports__",
      // ... others ...
    }
  }
}
```

## Template Definitions

### STARTUP_ENTRYPOINT.X - Synchronous (startup_entrypoint.ejs)

```javascript
__webpack_require__.X = function(result, chunkIds, fn) {
  // arguments: chunkIds, moduleId are deprecated
  var moduleId = chunkIds;
  if (!fn) chunkIds = result, fn = function() { 
    return __webpack_require__(__webpack_require__.s = moduleId);
  }
  chunkIds.map(__webpack_require__.e, __webpack_require__)
  var r = fn();
  return r === undefined ? result : r;
}
```

### STARTUP_ENTRYPOINT.X - Asynchronous (startup_entrypoint_with_async.ejs)

```javascript
__webpack_require__.X = function(result, chunkIds, fn) {
  // arguments: chunkIds, moduleId are deprecated
  var moduleId = chunkIds;
  if (!fn) chunkIds = result, fn = function() { 
    return __webpack_require__(__webpack_require__.s = moduleId);
  }
  return Promise.all(chunkIds.map(__webpack_require__.e, __webpack_require__)).then(function() {
      var r = fn();
      return r === undefined ? result : r;
  });
}
```

### ON_CHUNKS_LOADED.O - Deferred (on_chunk_loaded.ejs)

```javascript
var deferred = [];
__webpack_require__.O = function(result, chunkIds, fn, priority) {
  if (chunkIds) {
    // REGISTER phase: add to queue
    priority = priority || 0;
    for (var i = deferred.length; i > 0 && deferred[i - 1][2] > priority; i--)
      deferred[i] = deferred[i - 1];
    deferred[i] = [chunkIds, fn, priority];
    return;
  }
  // EXECUTE phase: process fulfilled handlers
  var notFulfilled = Infinity;
  for (var i = 0; i < deferred.length; i++) {
    var [chunkIds, fn, priority] = deferred[i];
    var fulfilled = true;
    for (var j = 0; j < chunkIds.length; j++) {
      if ((priority & 1 === 0 || notFulfilled >= priority) &&
          Object.keys(__webpack_require__.O).every(key => 
            __webpack_require__.O[key](chunkIds[j]))) {
        chunkIds.splice(j--, 1);
      } else {
        fulfilled = false;
        if (priority < notFulfilled) notFulfilled = priority;
      }
    }
    if (fulfilled) {
      deferred.splice(i--, 1);
      var r = fn();
      if (r !== undefined) result = r;
    }
  }
  return result;
};
```

## Inline Bailout Checks (mod.rs:247-402)

All these set `allow_inline_startup = false`:

```rust
// 1. Module factories used
if allow_inline_startup && module_factories {
  startup.push("// module factories are used so entry inlining is disabled".into());
  allow_inline_startup = false;
}

// 2. Module cache used
if allow_inline_startup && module_cache {
  startup.push("// module cache are used so entry inlining is disabled".into());
  allow_inline_startup = false;
}

// 3. Module execution intercepted
if allow_inline_startup && intercept_module_execution {
  startup.push("// module execution is intercepted so entry inlining is disabled".into());
  allow_inline_startup = false;
}

// 4. Entry depends on other chunks
if allow_inline_startup && !chunk_ids.is_empty() {
  buf2.push("// This entry module depends on other loaded chunks and execution need to be delayed".into());
  allow_inline_startup = false;
}

// 5. Entry referenced by other modules
if allow_inline_startup && {
  // Complex check for incoming connections
  module_graph.get_incoming_connections_by_origin_module(module)
    .iter().any(|(origin_module, connections)| { ... })
} {
  buf2.push("// This entry module is referenced by other modules so it can't be inlined".into());
  allow_inline_startup = false;
}

// 6. No top-level declarations
if allow_inline_startup && {
  let top_level_decls = codegen.data.get::<CodeGenerationDataTopLevelDeclarations>();
  top_level_decls.is_none()
} {
  buf2.push("// This entry module doesn't tell about it's top-level declarations so it can't be inlined".into());
  allow_inline_startup = false;
}

// 7. Hook bailout
let bailout = hooks.inline_in_runtime_bailout.call(compilation).await?;
if allow_inline_startup && let Some(bailout) = bailout {
  buf2.push(format!("// This entry module can't be inlined because {bailout}").into());
  allow_inline_startup = false;
}

// 8. Entry requires 'module' global
if allow_inline_startup
  && let Some(entry_runtime_requirements) = entry_runtime_requirements
  && entry_runtime_requirements.contains(RuntimeGlobals::MODULE)
{
  allow_inline_startup = false;
  buf2.push("// This entry module used 'module' so it can't be inlined".into());
}
```

## Complete File Locations

| Code | File | Lines | Purpose |
|------|------|-------|---------|
| render_bootstrap entry | rspack_plugin_javascript/src/plugin/mod.rs | 227 | Function signature |
| STARTUP_NO_DEFAULT check | rspack_plugin_javascript/src/plugin/mod.rs | 325-530 | Primary decision |
| Bailout checks | rspack_plugin_javascript/src/plugin/mod.rs | 247-412 | Inline optimization |
| Module execution logic | rspack_plugin_javascript/src/plugin/mod.rs | 421-484 | Three cases |
| ON_CHUNKS_LOADED post-proc | rspack_plugin_javascript/src/plugin/mod.rs | 486-493 | Deferred handling |
| STARTUP wrapping | rspack_plugin_javascript/src/plugin/mod.rs | 495-521 | Path A1a/A1b decision |
| Final startup paths | rspack_plugin_javascript/src/plugin/mod.rs | 531-545 | B and C paths |
| generate_entry_startup | rspack_plugin_runtime/src/helpers.rs | 181-273 | Helper function |
| STARTUP_ENTRYPOINT assignment | rspack_plugin_runtime/src/array_push_callback_chunk_format.rs | 56-60 | MF async flag |
| STARTUP assignment | rspack_plugin_runtime/src/startup_chunk_dependencies.rs | 25-47 | Dependent chunks |
| Runtime globals | rspack_core/src/runtime_globals.rs | 185, 201, 320, 328 | Definitions |

