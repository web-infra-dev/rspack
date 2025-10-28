# RSpack Runtime Code Templates Research

## Summary

RSpack uses a **RuntimeTemplate** system based on the **Dojang** templating engine (similar to EJS) to generate JavaScript code at compile time. This is the equivalent of webpack's RuntimeTemplate system.

---

## 1. RuntimeTemplate Equivalent in RSpack

**YES - Full equivalent exists at:**
- `/crates/rspack_core/src/runtime_template.rs`

### Key Components:

```rust
pub struct RuntimeTemplate {
  environment: Environment,
  dojang: Dojang,  // Template engine
}
```

### How It Works:
- Uses **Dojang** (Rust-based template engine with JS-like syntax)
- Registers helper functions for code generation
- Renders EJS-style templates with JSON parameters
- Supports environment-specific code generation (arrow functions, destructuring, etc.)

### Usage Pattern:
```rust
// Create runtime template
let rt = RuntimeTemplate::new(environment);

// Render a template with parameters
let code = compilation.runtime_template.render(
    "webpack/runtime/startup_entrypoint",
    Some(serde_json::json!({
        "_body": "return next();"
    }))
)?;
```

---

## 2. Helper Functions Implementation

**All helper functions are implemented in `RuntimeTemplate`:**

File: `/crates/rspack_core/src/runtime_template.rs`

### Available Helper Functions:

```rust
// Register based on environment capabilities
pub fn new(environment: Environment) -> Self {
    // Arrow function version (if supports_arrow_function())
    dojang.add_function_2("returningFunction", returning_function_arrow)
    dojang.add_function_1("basicFunction", basic_function_arrow)
    dojang.add_function_2("expressionFunction", expression_function_arrow)
    dojang.add_function_0("emptyFunction", empty_function_arrow)
    
    // Or traditional function version
    dojang.add_function_2("returningFunction", returning_function)
    dojang.add_function_1("basicFunction", basic_function)
    // ...
}
```

### Function Implementations:

#### returningFunction
```rust
fn returning_function_arrow(return_value: Operand, args: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    "({}) => ({})",
    join_to_string(&args, ", "),
    to_string(&return_value)
  )))
}

// Usage in template:
// <%- returningFunction("myValue", "arg1, arg2") %>
// Output: (arg1, arg2) => (myValue)
```

#### basicFunction
```rust
fn basic_function_arrow(args: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    r#"({}) =>"#,
    join_to_string(&args, ", ")
  )))
}

// Usage in template:
// <%- basicFunction("result, chunkIds, fn") %>
// Output: (result, chunkIds, fn) =>
```

#### expressionFunction
```rust
fn expression_function_arrow(expression: Operand, args: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    "({}) => ({})",
    join_to_string(&args, ", "),
    to_string(&expression)
  )))
}

// Usage in template:
// <%- expressionFunction("d[webpackExports]", "d") %>
// Output: (d) => (d[webpackExports])
```

#### emptyFunction
```rust
fn empty_function_arrow() -> Operand {
  Operand::Value(Value::from("function() {}"))
}
```

#### destructureArray
```rust
fn array_destructure(items: Operand, value: Operand) -> Operand {
  Operand::Value(Value::from(format!(
    "var [{}] = {};",
    join_to_string(&items, ", "),
    to_string(&value)
  )))
}

// Usage in template:
// <%- destructureArray("a, b, c", "arr") %>
// Output: var [a, b, c] = arr;
```

### Helper Methods on RuntimeTemplate:

```rust
impl RuntimeTemplate {
  // Method for generating returning functions programmatically
  pub fn returning_function(&self, return_value: &str, args: &str) -> String {
    if self.environment.supports_arrow_function() {
      format!("({args}) => ({return_value})")
    } else {
      format!("function({args}) {{ return {return_value}; }}")
    }
  }

  pub fn basic_function(&self, args: &str, body: &str) -> String {
    if self.environment.supports_arrow_function() {
      format!("({args}) => {{\n {body} \n}}")
    } else {
      format!("function({args}) {{\n {body} \n}}")
    }
  }
}
```

---

## 3. Promise.all Pattern Utilities

**Promise.all patterns are generated within templates using basic functions.**

### Files Using Promise.all:

1. **startup_entrypoint_with_async.ejs:**
```ejs
<%- STARTUP_ENTRYPOINT %> = <%- basicFunction("result, chunkIds, fn") %> {
  // arguments: chunkIds, moduleId are deprecated
  var moduleId = chunkIds;
  if (!fn) chunkIds = result, fn = <%- returningFunction("__webpack_require__(__webpack_require__.s = moduleId)", "") %>
  return Promise.all(chunkIds.map(<%- ENSURE_CHUNK %>, <%- REQUIRE %>)).then(<%- basicFunction("") %> {
      var r = fn();
      return r === undefined ? result : r;
  });
}
```

2. **async_module.ejs:**
```ejs
Promise.all(asyncDeps.map(__webpack_require__)).then(<%- returningFunction("callback(d)", "") %>)
```

3. **startup_chunk_dependencies.rs (Rust code generation):**
```rust
// From: /crates/rspack_plugin_runtime/src/runtime_module/startup_chunk_dependencies.rs
let body = if self.async_chunk_loading {
    match chunk_ids.len() {
        1 => format!(
            r#"return {}("{}").then(next);"#,
            RuntimeGlobals::ENSURE_CHUNK,
            chunk_ids.first().expect("Should has at least one chunk")
        ),
        2 => format!(
            r#"return Promise.all([{}]).then(next);"#,
            chunk_ids
              .iter()
              .map(|cid| format!(r#"{}("{}")"#, RuntimeGlobals::ENSURE_CHUNK, cid))
              .join(",\n")
        ),
        _ => format!(
            r#"return Promise.all({}.map({}, {})).then(next);"#,
            serde_json::to_string(&chunk_ids).expect("Invalid json to string"),
            RuntimeGlobals::ENSURE_CHUNK,
            RuntimeGlobals::REQUIRE
        ),
    }
} else {
    // Synchronous version
}
```

### Pattern: Wrapping with Promise.all

The pattern for creating Promise.all wrappers:

1. **Single chunk:** Direct ensure call with `.then()`
2. **Two chunks:** `Promise.all([chunk1(), chunk2()]).then(...)`
3. **Multiple chunks:** `Promise.all(chunks.map(ENSURE_CHUNK, REQUIRE)).then(...)`

---

## 4. Template.asString Equivalent

**There is NO direct `Template.asString()` equivalent in rspack.**

Instead, rspack uses:

### String Building Approaches:

1. **Direct String Concatenation:**
```rust
let mut source = String::new();
source.push_str(&format!("var x = {};\n", some_value));
source.push_str("more code\n");
```

2. **ConcatSource (RawStringSource):**
```rust
use rspack_sources::{RawStringSource, ConcatSource};

let mut source = ConcatSource::default();
source.add(RawStringSource::from("var x = 1;\n"));
source.add(RawStringSource::from("var y = 2;\n"));
```

3. **Format Macros with line joining:**
```rust
chunk_ids
  .iter()
  .map(|cid| format!(r#"{}("{}")"#, RuntimeGlobals::ENSURE_CHUNK, cid))
  .join(",\n")  // Join with newlines instead of .asString()
```

---

## 5. Code Generation Utilities

### Key Utility Functions:

#### From `/crates/rspack_plugin_runtime/src/helpers.rs`:

```rust
pub fn stringify_chunks_to_array(chunks: &HashSet<ChunkId>) -> String {
  // Converts chunk IDs to array format: [1, 2, 3]
  let mut v = Vec::from_iter(chunks.iter());
  v.sort_unstable();
  format!(r#"[{}]"#, ...)
}

pub fn generate_entry_startup(
  compilation: &Compilation,
  chunk: &ChunkUkey,
  entries: &IdentifierLinkedMap<ChunkGroupUkey>,
  passive: bool,
) -> BoxSource {
  // Generates startup code for entry points
  let mut source = String::default();
  source.push_str(&format!(
    "var __webpack_exec__ = function(moduleId) {{ return __webpack_require__({} = moduleId) }}\n",
    RuntimeGlobals::ENTRY_MODULE_ID
  ));
  // ... more code generation
  RawStringSource::from(source).boxed()
}
```

#### From `/crates/rspack_plugin_runtime/src/runtime_module/utils.rs`:

```rust
pub fn stringify_chunks(chunks: &HashSet<ChunkId>, value: u8) -> String {
  // Stringify chunk map: {1: 1, 2: 1}
}

pub fn stringify_dynamic_chunk_map<F>(
  f: F,
  chunks: &UkeyIndexSet<ChunkUkey>,
  chunk_map: &UkeyIndexMap<ChunkUkey, &Chunk>,
  compilation: &Compilation,
) -> String {
  // Generate dynamic chunk lookup maps
}

pub fn chunk_has_js(chunk_ukey: &ChunkUkey, compilation: &Compilation) -> bool {
  // Check if chunk contains JS modules
}
```

---

## 6. Template Helper Location in rspack_core

File: `/crates/rspack_core/src/utils/template.rs`

```rust
pub fn to_normal_comment(str: &str) -> String {
  if str.is_empty() {
    return String::new();
  }
  format!("/* {} */", str.cow_replace("*/", "* /"))
}
```

---

## 7. How to Generate Wrapped Startup Code - Examples

### Example 1: Async Startup with Promise.all

**Using RuntimeTemplate directly:**

```rust
// From StartupChunkDependenciesRuntimeModule
impl RuntimeModule for StartupChunkDependenciesRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/startup_chunk_dependencies.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> Result<String> {
    // Get all chunks that need to be loaded
    let chunk_ids = compilation
      .chunk_graph
      .get_chunk_entry_dependent_chunks_iterable(&chunk_ukey, ...)
      .map(|chunk_ukey| {
        compilation.chunk_by_ukey
          .expect_get(&chunk_ukey)
          .expect_id(&compilation.chunk_ids_artifact)
          .to_string()
      })
      .collect::<Vec<_>>();

    // Generate async body
    let body = match chunk_ids.len() {
      1 => format!(
        r#"return {}("{}").then(next);"#,
        RuntimeGlobals::ENSURE_CHUNK,
        chunk_ids.first().unwrap()
      ),
      2 => format!(
        r#"return Promise.all([{}]).then(next);"#,
        chunk_ids
          .iter()
          .map(|cid| format!(r#"{}("{}")"#, RuntimeGlobals::ENSURE_CHUNK, cid))
          .join(",\n")
      ),
      _ => format!(
        r#"return Promise.all({}.map({}, {})).then(next);"#,
        serde_json::to_string(&chunk_ids).unwrap(),
        RuntimeGlobals::ENSURE_CHUNK,
        RuntimeGlobals::REQUIRE
      ),
    };

    // Render the template with the body
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_body": body,
      })),
    )?;

    Ok(source)
  }
}
```

### Example 2: Startup Entrypoint with Async Wrapper

**Template file (startup_entrypoint_with_async.ejs):**

```ejs
<%- STARTUP_ENTRYPOINT %> = <%- basicFunction("result, chunkIds, fn") %> {
  // arguments: chunkIds, moduleId are deprecated
  var moduleId = chunkIds;
  if (!fn) chunkIds = result, fn = <%- returningFunction("__webpack_require__(__webpack_require__.s = moduleId)", "") %>
  return Promise.all(chunkIds.map(<%- ENSURE_CHUNK %>, <%- REQUIRE %>)).then(<%- basicFunction("") %> {
      var r = fn();
      return r === undefined ? result : r;
  });
}
```

**Generated output:**

```javascript
__webpack_startup_entrypoint__ = function(result, chunkIds, fn) {
  // arguments: chunkIds, moduleId are deprecated
  var moduleId = chunkIds;
  if (!fn) chunkIds = result, fn = (moduleId) => (__webpack_require__(__webpack_require__.s = moduleId))
  return Promise.all(chunkIds.map(__webpack_require_ensure__, __webpack_require__)).then(function() {
      var r = fn();
      return r === undefined ? result : r;
  });
}
```

### Example 3: Async Module with Promise Wrapping

**Template snippet from async_module.ejs:**

```ejs
var wrapDeps = <%- basicFunction("deps") %> {
	return deps.map(<%- basicFunction("dep") %> {
		if (dep !== null && typeof dep === "object") {
			if(!dep[webpackQueues] && dep[webpackDefer]) {
				var asyncDeps = dep[webpackDefer];
				// ... checks ...
				if (hasUnresolvedAsyncSubgraph) {
					var d = dep;
					dep = {
						then(callback) {
							Promise.all(asyncDeps.map(__webpack_require__)).then(<%- returningFunction("callback(d)", "") %>)
						}
					};
				}
			}
		}
		// ...
	});
};
```

**Generated output:**

```javascript
var wrapDeps = function(deps) {
	return deps.map(function(dep) {
		if (dep !== null && typeof dep === "object") {
			if(!dep[webpackQueues] && dep[webpackDefer]) {
				var asyncDeps = dep[webpackDefer];
				// ... checks ...
				if (hasUnresolvedAsyncSubgraph) {
					var d = dep;
					dep = {
						then(callback) {
							Promise.all(asyncDeps.map(__webpack_require__)).then((d) => (callback(d)))
						}
					};
				}
			}
		}
		// ...
	});
};
```

---

## Summary Table: Helper Functions

| Function | Input | Output |
|----------|-------|--------|
| `returningFunction(expr, args)` | `"myVal", "arg1, arg2"` | `(arg1, arg2) => (myVal)` |
| `basicFunction(args)` | `"arg1, arg2"` | `(arg1, arg2) =>` |
| `expressionFunction(expr, args)` | `"x + y", "x, y"` | `(x, y) => (x + y)` |
| `emptyFunction()` | - | `function() {}` |
| `destructureArray(items, val)` | `"a, b", "arr"` | `var [a, b] = arr;` |

---

## Key Files Reference

1. **RuntimeTemplate Implementation:**
   - `/crates/rspack_core/src/runtime_template.rs`

2. **Runtime Module Implementations:**
   - `/crates/rspack_plugin_runtime/src/runtime_module/startup_entrypoint.rs`
   - `/crates/rspack_plugin_runtime/src/runtime_module/startup_chunk_dependencies.rs`

3. **Templates (EJS files):**
   - `/crates/rspack_plugin_runtime/src/runtime_module/runtime/startup_entrypoint.ejs`
   - `/crates/rspack_plugin_runtime/src/runtime_module/runtime/startup_entrypoint_with_async.ejs`
   - `/crates/rspack_plugin_runtime/src/runtime_module/runtime/startup_chunk_dependencies.ejs`
   - `/crates/rspack_plugin_runtime/src/runtime_module/runtime/async_module.ejs`

4. **Helper Functions:**
   - `/crates/rspack_plugin_runtime/src/helpers.rs`
   - `/crates/rspack_plugin_runtime/src/runtime_module/utils.rs`

5. **Source Generation:**
   - `/crates/rspack_core/src/utils/template.rs`
