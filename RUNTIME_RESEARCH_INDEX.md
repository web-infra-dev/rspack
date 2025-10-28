# RSpack Runtime Code Templates Research - Complete Index

## Overview

This research covers how RSpack generates runtime code using the **RuntimeTemplate system**, which is the equivalent of webpack's RuntimeTemplate but implemented in Rust using the **Dojang templating engine**.

---

## Three Key Documents

### 1. RUNTIME_TEMPLATES_SUMMARY.md (START HERE)
**Length:** Quick read (4 pages)
**Best for:** Getting the gist of the system

This is the executive summary covering:
- What RuntimeTemplate is
- The helper functions available
- Promise.all generation patterns
- Quick implementation template
- Key file references

**Read this first** to understand the architecture.

### 2. RUNTIME_TEMPLATE_RESEARCH.md (DETAILED REFERENCE)
**Length:** Comprehensive (15 pages)
**Best for:** Deep technical understanding

Complete technical reference including:
- RuntimeTemplate structure and how it works
- All 5 helper functions with implementation code
- Promise.all patterns in detail
- Code generation utilities
- 3 complete working examples with input/output
- Summary table of all functions
- Full file reference list

**Read this** when you need to understand the implementation details.

### 3. RUNTIME_CODE_GENERATION_GUIDE.md (PRACTICAL GUIDE)
**Length:** Implementation focused (13 pages)
**Best for:** Actually writing code

Practical implementation guide with:
- Step-by-step patterns for common tasks
- 7 pattern library examples
- How to access compilation context
- Complete working example (Async Startup Module)
- Tips and best practices
- Debugging strategies

**Read this** when you're implementing new runtime modules.

---

## Quick Navigation

### I want to understand the architecture
Start with: **RUNTIME_TEMPLATES_SUMMARY.md**

### I want technical details about how things work
Read: **RUNTIME_TEMPLATE_RESEARCH.md** (sections 1-6)

### I'm building a new async startup implementation
Use: **RUNTIME_CODE_GENERATION_GUIDE.md** (sections 1-3)

### I want to see actual working code
Check: **RUNTIME_TEMPLATE_RESEARCH.md** (section 7) or **RUNTIME_CODE_GENERATION_GUIDE.md** (Example section)

---

## Key Findings Summary

### Question 1: Is there a RuntimeTemplate equivalent?
**YES** - Located at `/crates/rspack_core/src/runtime_template.rs`

### Question 2: How are helper functions implemented?
In `RuntimeTemplate::new()`, functions are registered with Dojang:
- `returningFunction(return_value, args)` - Creates arrow or function returning a value
- `basicFunction(args)` - Creates function signature
- `expressionFunction(expr, args)` - Creates arrow/function with inline expression
- `emptyFunction()` - Creates `function() {}`
- `destructureArray(items, value)` - Creates destructuring or traditional assignment

### Question 3: Where are Promise.all utilities?
There are no special utilities. Promise.all is generated using:
- Basic function helpers
- String formatting in Rust
- Direct template code in EJS files

### Question 4: Does Template.asString exist?
**NO** - Not needed. RSpack uses:
- Direct String concatenation
- RawStringSource/ConcatSource for efficient building
- `.join()` methods for array processing

### Question 5: What utilities exist for code generation?
Key utilities in `/crates/rspack_plugin_runtime/src/`:
- `helpers.rs` - `generate_entry_startup()`, `stringify_chunks_to_array()`
- `runtime_module/utils.rs` - `stringify_chunks()`, `stringify_dynamic_chunk_map()`, `chunk_has_js()`

### Question 6: Template helpers in rspack_core
Located in `/crates/rspack_core/src/utils/template.rs`:
- `to_normal_comment(str)` - Wraps text in comment syntax

---

## How to Generate Wrapped Startup Code

### Pattern 1: Single Chunk
```rust
format!(r#"return {}("{}").then(next);"#, RuntimeGlobals::ENSURE_CHUNK, chunk_id)
```

### Pattern 2: Multiple Chunks
```rust
let promises = chunk_ids.iter()
  .map(|id| format!(r#"{}("{}")"#, RuntimeGlobals::ENSURE_CHUNK, id))
  .join(",\n");
format!(r#"return Promise.all([{}]).then(next);"#, promises)
```

### Pattern 3: Dynamic Chunks
```rust
let json = serde_json::to_string(&chunk_ids)?;
format!(r#"return Promise.all({}.map({}, {})).then(next);"#, json, RuntimeGlobals::ENSURE_CHUNK, RuntimeGlobals::REQUIRE)
```

### Pattern 4: Template Helper
```ejs
<%- returningFunction("callback(result)", "") %>
```
Generates: `(result) => (callback(result))` or `function(result) { return callback(result); }`

---

## Real-World Example: From the Codebase

This is how RSpack handles async startup chunk dependencies:

**File:** `/crates/rspack_plugin_runtime/src/runtime_module/startup_chunk_dependencies.rs`

```rust
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
};

let source = compilation.runtime_template.render(
    &self.id,
    Some(serde_json::json!({
        "_body": body,
    })),
)?;
```

This is the **exact pattern** you should use for async startup implementation.

---

## File References Used in Research

### Core Runtime Template
- `/crates/rspack_core/src/runtime_template.rs` - Main RuntimeTemplate implementation
- `/crates/rspack_core/src/utils/template.rs` - Template utilities

### Runtime Modules (Examples)
- `/crates/rspack_plugin_runtime/src/runtime_module/startup_entrypoint.rs` - Startup entrypoint module
- `/crates/rspack_plugin_runtime/src/runtime_module/startup_chunk_dependencies.rs` - Chunk dependencies
- `/crates/rspack_plugin_runtime/src/helpers.rs` - Entry startup generation helpers

### Template Files (EJS)
- `/crates/rspack_plugin_runtime/src/runtime_module/runtime/startup_entrypoint.ejs` - Sync startup
- `/crates/rspack_plugin_runtime/src/runtime_module/runtime/startup_entrypoint_with_async.ejs` - Async startup
- `/crates/rspack_plugin_runtime/src/runtime_module/runtime/startup_chunk_dependencies.ejs` - Chunk dependencies
- `/crates/rspack_plugin_runtime/src/runtime_module/runtime/async_module.ejs` - Async module handling

### Utilities
- `/crates/rspack_plugin_runtime/src/runtime_module/utils.rs` - Chunk utility functions
- `/crates/rspack_plugin_javascript/src/runtime.rs` - JavaScript runtime utilities

---

## For Your Async Startup Runtime Promise Implementation

The most relevant files for your work are:

1. **Study these modules:**
   - `/crates/rspack_plugin_runtime/src/runtime_module/startup_chunk_dependencies.rs` (Shows Promise pattern)
   - `/crates/rspack_plugin_runtime/src/runtime_module/startup_entrypoint.rs` (Shows template switching)

2. **Use these templates as reference:**
   - `/crates/rspack_plugin_runtime/src/runtime_module/runtime/startup_entrypoint_with_async.ejs`
   - `/crates/rspack_plugin_runtime/src/runtime_module/runtime/async_module.ejs`

3. **Reference these utilities:**
   - `RuntimeGlobals::ENSURE_CHUNK`
   - `RuntimeGlobals::STARTUP_ENTRYPOINT`
   - `RuntimeGlobals::REQUIRE`

4. **Follow this pattern:**
   - Create RuntimeModule that implements `template()` and `generate()`
   - In `generate()`, build the body string with formatted Promise.all chains
   - Call `compilation.runtime_template.render()` with the body as parameter
   - Use helper functions like `returningFunction()` and `basicFunction()` in templates

---

## Technology Stack

- **Template Engine:** Dojang (Rust-based, similar to EJS)
- **Runtime Globals:** RuntimeGlobals enum (located in rspack_core)
- **Source Building:** RawStringSource, ConcatSource (from rspack_sources)
- **Code Generation:** Rust String formatting + serde_json
- **Environment Support:** Conditional code generation based on target environment capabilities

---

## Document Statistics

- **Total Documentation:** 42 KB across 3 files
- **Code Examples:** 25+ working examples
- **File References:** 20+ key files identified
- **Pattern Library:** 7 common patterns documented
- **Implementation Examples:** 3 complete working examples

---

## How These Documents Were Created

This research was conducted by:
1. Grepping for RuntimeTemplate across the codebase
2. Reading runtime_template.rs to understand the architecture
3. Analyzing 4 different runtime modules for patterns
4. Examining 4 EJS template files for Promise.all usage
5. Extracting helper functions and their implementations
6. Tracing through existing code that generates Promise.all chains
7. Documenting patterns from actual working code

All examples are from the actual RSpack codebase, not synthetic examples.

---

## Last Updated

October 27, 2025

These documents can be used as reference material for anyone working on:
- Async startup runtime promises
- Promise.all pattern generation
- Runtime code generation
- Template-based code creation in RSpack
