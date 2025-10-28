# RSpack Runtime Templates - Executive Summary

## What You Need to Know

### 1. RuntimeTemplate Exists and Works Like Webpack
RSpack has a **complete RuntimeTemplate equivalent** located at:
- `/crates/rspack_core/src/runtime_template.rs`

It uses the **Dojang templating engine** (similar to EJS) instead of webpack's JavaScript template engine.

### 2. Helper Functions Are Available
All needed helper functions are implemented:
- `returningFunction(return_expr, args)` - Creates arrow or traditional function that returns a value
- `basicFunction(args)` - Creates function signature
- `expressionFunction(expr, args)` - Creates inline expression function
- `emptyFunction()` - Creates empty function
- `destructureArray(items, array)` - Creates array destructuring or traditional assignment

### 3. Promise.all Generation Works
Promise.all patterns are generated in two ways:

**In Rust code:**
```rust
match chunk_ids.len() {
  1 => format!(r#"return {}("{}").then(next);"#, ENSURE_CHUNK, id),
  2 => format!(r#"return Promise.all([{}]).then(next);"#, promises.join(",")),
  _ => format!(r#"return Promise.all({}.map({}, {})).then(next);"#, json_array, ENSURE_CHUNK, REQUIRE),
}
```

**In EJS templates:**
```ejs
return Promise.all(chunkIds.map(<%- ENSURE_CHUNK %>, <%- REQUIRE %>)).then(<%- basicFunction("") %> {
    // callback
});
```

### 4. No asString() - Use String Builders Instead
RSpack uses:
- Direct String concatenation
- RawStringSource/ConcatSource for efficient building
- `.join()` method for arrays

### 5. Key Utility Functions

| Location | Function | Purpose |
|----------|----------|---------|
| `/helpers.rs` | `generate_entry_startup()` | Generate startup code for entries |
| `/helpers.rs` | `stringify_chunks_to_array()` | Convert chunk IDs to array format |
| `/runtime_module/utils.rs` | `stringify_chunks()` | Create chunk lookup maps |
| `/runtime_module/utils.rs` | `chunk_has_js()` | Check if chunk has JS modules |

---

## Quick Implementation Template

### Step 1: Create RuntimeModule

```rust
#[impl_runtime_module]
#[derive(Debug)]
pub struct MyRuntimeModule {
  id: Identifier,
}

#[async_trait::async_trait]
impl RuntimeModule for MyRuntimeModule {
  fn name(&self) -> Identifier { self.id }
  
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/my_template.ejs").to_string(),
    )]
  }
  
  async fn generate(&self, compilation: &Compilation) -> Result<String> {
    compilation.runtime_template.render(&self.id, None)
  }
}
```

### Step 2: Create EJS Template

```ejs
<%- MY_VAR %> = <%- basicFunction("chunks") %> {
  return Promise.all(chunks.map(<%- ENSURE_CHUNK %>, <%- REQUIRE %>)).then(<%- basicFunction("") %> {
    // callback
  });
}
```

### Step 3: Render with Parameters

```rust
let body = format!(
  r#"return Promise.all([{}]).then(next);"#,
  chunk_ids.iter()
    .map(|id| format!(r#"{}("{}")"#, RuntimeGlobals::ENSURE_CHUNK, id))
    .join(",\n")
);

compilation.runtime_template.render(
  &self.id,
  Some(serde_json::json!({ "_body": body }))
)?
```

---

## Key Files to Reference

1. **RuntimeTemplate Core:**
   - `/crates/rspack_core/src/runtime_template.rs` - Main implementation

2. **Existing Examples:**
   - `/crates/rspack_plugin_runtime/src/runtime_module/startup_entrypoint.rs`
   - `/crates/rspack_plugin_runtime/src/runtime_module/startup_chunk_dependencies.rs`

3. **Template Examples:**
   - `/crates/rspack_plugin_runtime/src/runtime_module/runtime/startup_entrypoint_with_async.ejs`
   - `/crates/rspack_plugin_runtime/src/runtime_module/runtime/async_module.ejs`

4. **Utilities:**
   - `/crates/rspack_plugin_runtime/src/helpers.rs`
   - `/crates/rspack_plugin_runtime/src/runtime_module/utils.rs`

---

## For Your Async Startup Implementation

You can follow the exact pattern used in:
- `StartupChunkDependenciesRuntimeModule` - Shows Promise.all generation
- `StartupEntrypointRuntimeModule` - Shows async/sync template switching

Both are located in `/crates/rspack_plugin_runtime/src/runtime_module/`

The templates demonstrate how to:
1. Generate Promise.all chains for multiple chunks
2. Handle single vs multiple chunks differently
3. Use RuntimeGlobals for consistent naming
4. Wrap startup code asynchronously

---

## Two Documentation Files Created

1. **RUNTIME_TEMPLATE_RESEARCH.md** - Complete technical reference with all details
2. **RUNTIME_CODE_GENERATION_GUIDE.md** - Practical implementation guide with patterns

Both are saved in the repository root for reference.
