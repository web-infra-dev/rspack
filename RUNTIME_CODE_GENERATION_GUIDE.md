# RSpack Runtime Code Generation Guide

## Quick Start: How to Use RuntimeTemplate for Wrapped Startup Code

### 1. Basic Pattern: Create a RuntimeModule

```rust
use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeModule, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct MyStartupRuntimeModule {
  id: Identifier,
  async_enabled: bool,
}

impl MyStartupRuntimeModule {
  pub fn new(async_enabled: bool) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/my_startup"),
      async_enabled,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for MyStartupRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    // Return templates to register with RuntimeTemplate
    vec![(
      self.id.to_string(),
      include_str!("runtime/my_startup.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> Result<String> {
    // This is where you generate the runtime code
    compilation.runtime_template.render(&self.id, None)
  }
}
```

### 2. Create Your Template (my_startup.ejs)

```ejs
<%- MY_STARTUP_VAR %> = <%- basicFunction("chunks") %> {
  // My startup logic here
  return Promise.all(chunks.map(<%- ENSURE_CHUNK %>, <%- REQUIRE %>)).then(<%- basicFunction("") %> {
    console.log("All chunks loaded!");
  });
}
```

### 3. Rendering with Parameters

```rust
async fn generate(&self, compilation: &Compilation) -> Result<String> {
  let chunk_ids = vec!["1", "2", "3"];
  
  let body = format!(
    r#"return Promise.all([{}]).then(next);"#,
    chunk_ids
      .iter()
      .map(|id| format!(r#"{}("{}")"#, RuntimeGlobals::ENSURE_CHUNK, id))
      .join(",\n")
  );

  compilation.runtime_template.render(
    &self.id,
    Some(serde_json::json!({
      "_body": body,
    }))
  )
}
```

---

## Pattern Library: Common Startup Code Patterns

### Pattern 1: Single Chunk Async Load

**Rust Code:**
```rust
format!(
  r#"return {}("{}").then(next);"#,
  RuntimeGlobals::ENSURE_CHUNK,
  chunk_id
)
```

**Generated JS:**
```javascript
return __webpack_require_ensure__("123").then(next);
```

### Pattern 2: Multiple Chunks with Promise.all

**Rust Code:**
```rust
let promises = chunk_ids
  .iter()
  .map(|id| format!(r#"{}("{}")"#, RuntimeGlobals::ENSURE_CHUNK, id))
  .collect::<Vec<_>>();

format!(
  r#"return Promise.all([{}]).then(next);"#,
  promises.join(",\n")
)
```

**Generated JS:**
```javascript
return Promise.all([
  __webpack_require_ensure__("1"),
  __webpack_require_ensure__("2"),
  __webpack_require_ensure__("3")
]).then(next);
```

### Pattern 3: Dynamic Array Mapping

**Rust Code:**
```rust
let chunks_json = serde_json::to_string(&chunk_ids).unwrap();

format!(
  r#"return Promise.all({}.map({}, {})).then(next);"#,
  chunks_json,
  RuntimeGlobals::ENSURE_CHUNK,
  RuntimeGlobals::REQUIRE
)
```

**Generated JS:**
```javascript
return Promise.all(["1","2","3"].map(__webpack_require_ensure__, __webpack_require__)).then(next);
```

### Pattern 4: Function Wrapping with basicFunction

**Template:**
```ejs
<%- MY_FUNC %> = <%- basicFunction("arg1, arg2, arg3") %> {
  // Function body here
}
```

**Generated JS (with arrow function support):**
```javascript
__MY_FUNC__ = (arg1, arg2, arg3) => {
  // Function body here
}
```

**Generated JS (without arrow function support):**
```javascript
__MY_FUNC__ = function(arg1, arg2, arg2) {
  // Function body here
}
```

### Pattern 5: Returning Function

**Template:**
```ejs
var callback = <%- returningFunction("__webpack_require__(moduleId)", "moduleId") %>
```

**Generated JS (arrow):**
```javascript
var callback = (moduleId) => (__webpack_require__(moduleId))
```

**Generated JS (traditional):**
```javascript
var callback = function(moduleId) { return __webpack_require__(moduleId); }
```

### Pattern 6: Expression Functions

**Template:**
```ejs
deps.map(<%- expressionFunction("d[webpackExports]", "d") %>)
```

**Generated JS:**
```javascript
deps.map((d) => (d[webpackExports]))
```

### Pattern 7: Array Destructuring

**Template:**
```ejs
<%- destructureArray("a, b, c", "arr") %>
```

**Generated JS (with destructuring support):**
```javascript
var [a, b, c] = arr;
```

**Generated JS (without destructuring support):**
```javascript
var a = arr[0];
var b = arr[1];
var c = arr[2];
```

---

## Working with Compilation Context

### Accessing Chunk Information

```rust
async fn generate(&self, compilation: &Compilation) -> Result<String> {
  // Get chunk entry dependent chunks
  let chunk_ids = compilation
    .chunk_graph
    .get_chunk_entry_dependent_chunks_iterable(&chunk_ukey, &compilation.chunk_group_by_ukey)
    .map(|chunk_ukey| {
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
      chunk
        .expect_id(&compilation.chunk_ids_artifact)
        .to_string()
    })
    .collect::<Vec<_>>();

  Ok(chunk_ids)
}
```

### Checking for Chunk Types

```rust
use rspack_plugin_runtime::runtime_module::chunk_has_js;

let has_js = chunk_has_js(&chunk_ukey, compilation);
```

### Getting Chunk Filenames

```rust
use rspack_plugin_runtime::helpers::get_chunk_output_name;

let output_name = get_chunk_output_name(chunk, compilation).await?;
```

---

## Template Parameters (RuntimeGlobals)

These are automatically available in all templates:

```rust
// Available RuntimeGlobals:
RuntimeGlobals::REQUIRE              // __webpack_require__
RuntimeGlobals::ENSURE_CHUNK         // __webpack_require_ensure__
RuntimeGlobals::STARTUP              // __webpack_startup__
RuntimeGlobals::STARTUP_ENTRYPOINT   // __webpack_startup_entrypoint__
RuntimeGlobals::ENTRY_MODULE_ID      // __webpack_entry_module_id__
RuntimeGlobals::MODULE_CACHE         // __webpack_module_cache__
RuntimeGlobals::EXPORTS              // __webpack_exports__
RuntimeGlobals::ASYNC_MODULE         // __webpack_async_module__
RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT_SYMBOL
```

---

## Example: Complete Async Startup Module

### 1. Rust Module (async_startup.rs)

```rust
use std::iter;
use itertools::Itertools;
use rspack_collections::Identifier;
use rspack_core::{ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct AsyncStartupRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl AsyncStartupRuntimeModule {
  pub fn new() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/async_startup"), None)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for AsyncStartupRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/async_startup.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    if let Some(chunk_ukey) = self.chunk {
      // Get all chunks that need loading
      let chunk_ids = compilation
        .chunk_graph
        .get_chunk_entry_dependent_chunks_iterable(
          &chunk_ukey,
          &compilation.chunk_group_by_ukey,
        )
        .map(|chunk_ukey| {
          compilation
            .chunk_by_ukey
            .expect_get(&chunk_ukey)
            .expect_id(&compilation.chunk_ids_artifact)
            .to_string()
        })
        .collect::<Vec<_>>();

      // Generate the body based on chunk count
      let body = match chunk_ids.len() {
        0 => "return Promise.resolve();".to_string(),
        1 => format!(
          r#"return {}("{}").then(next);"#,
          RuntimeGlobals::ENSURE_CHUNK,
          chunk_ids[0]
        ),
        _ => {
          let promise_calls = chunk_ids
            .iter()
            .map(|id| format!(r#"{}("{}")"#, RuntimeGlobals::ENSURE_CHUNK, id))
            .join(",\n");
          format!(r#"return Promise.all([{}]).then(next);"#, promise_calls)
        }
      };

      // Render template with generated body
      let source = compilation.runtime_template.render(
        &self.id,
        Some(serde_json::json!({
          "_body": body,
        })),
      )?;

      Ok(source)
    } else {
      Err(rspack_error::error!(
        "AsyncStartupRuntimeModule requires chunk to be attached"
      ))
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
```

### 2. Template (async_startup.ejs)

```ejs
var next = <%- STARTUP %>
<%- STARTUP %> = <%- basicFunction("") %> {
  <%- _body %>
}
```

### 3. Generated Output (for 2 chunks)

```javascript
var next = __webpack_startup__
__webpack_startup__ = function() {
  return Promise.all([
    __webpack_require_ensure__("1"),
    __webpack_require_ensure__("2")
  ]).then(next);
}
```

---

## Tips & Best Practices

### 1. Use RuntimeGlobals Constants
Always use `RuntimeGlobals::*` instead of hardcoding `__webpack_...` strings:
```rust
// Good
format!(r#"{}()"#, RuntimeGlobals::ENSURE_CHUNK)

// Avoid
format!(r#"__webpack_require_ensure__()"#)
```

### 2. Handle Environment Differences
Check environment capabilities for syntax:
```rust
let code = if compilation.options.output.environment.supports_arrow_function() {
  "(x) => x"
} else {
  "function(x) { return x; }"
};
```

### 3. Use JSON Serialization for Values
Always properly serialize JavaScript values:
```rust
// Good
serde_json::to_string(&chunk_id).unwrap()

// Avoid
format!("\"{}\"", chunk_id)
```

### 4. Template Organization
Put reusable templates in separate `.ejs` files:
```rust
fn template(&self) -> Vec<(String, String)> {
  vec![
    (self.id.to_string(), include_str!("runtime/my_template.ejs").to_string()),
    (format!("{}_helper", self.id), include_str!("runtime/my_helper.ejs").to_string()),
  ]
}
```

### 5. Render with Proper Parameter Passing
Pass all needed parameters as JSON object:
```rust
compilation.runtime_template.render(
  &self.id,
  Some(serde_json::json!({
    "_body": body,
    "_chunk_ids": serde_json::to_string(&chunk_ids)?,
    "_count": chunk_ids.len(),
  }))
)?
```

---

## Debugging Template Generation

### Print Generated Code

```rust
let generated = compilation.runtime_template.render(&self.id, None)?;
eprintln!("Generated code:\n{}", generated);
```

### Validate JSON Parameters

```rust
let params = serde_json::json!({
  "_body": body,
});
eprintln!("Parameters: {}", serde_json::to_string_pretty(&params)?);
```

### Check Environment Capabilities

```rust
let env = &compilation.options.output.environment;
eprintln!("Arrow functions: {}", env.supports_arrow_function());
eprintln!("Destructuring: {}", env.supports_destructuring());
eprintln!("Async/await: {}", env.supports_async_function());
```

---

## References

- Main RuntimeTemplate: `/crates/rspack_core/src/runtime_template.rs`
- Runtime modules: `/crates/rspack_plugin_runtime/src/runtime_module/`
- Helpers: `/crates/rspack_plugin_runtime/src/helpers.rs`
- Template examples: `/crates/rspack_plugin_runtime/src/runtime_module/runtime/*.ejs`
