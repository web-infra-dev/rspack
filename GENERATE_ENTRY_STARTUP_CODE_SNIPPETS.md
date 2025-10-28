# Code Snippets: generate_entry_startup Complete Implementation

## Complete Function (helpers.rs:181-273)

```rust
pub fn generate_entry_startup(
  compilation: &Compilation,
  chunk: &ChunkUkey,
  entries: &IdentifierLinkedMap<ChunkGroupUkey>,
  passive: bool,
) -> BoxSource {
  let mut module_id_exprs = vec\![];
  let mut chunks_ids = HashSet::default();
  let module_graph = compilation.get_module_graph();
  
  // Phase 2: Extract module IDs and collect dependent chunks
  for (module, entry) in entries {
    // Get the module object and its ID
    if let Some(module_id) = module_graph
      .module_by_identifier(module)
      .filter(|module| {
        module
          .source_types(&module_graph)
          .contains(&SourceType::JavaScript)
      })
      .and_then(|module| {
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier())
      })
    {
      let module_id_expr = serde_json::to_string(module_id).expect("invalid module_id");
      module_id_exprs.push(module_id_expr);
    } else {
      continue;
    }

    // Collect all chunks needed by this entry
    if let Some(runtime_chunk) = compilation
      .chunk_group_by_ukey
      .get(entry)
      .map(|e| e.get_runtime_chunk(&compilation.chunk_group_by_ukey))
    {
      let chunks = get_all_chunks(
        entry,
        chunk,
        Some(&runtime_chunk),
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
  }

  // Phase 1: Create __webpack_exec__ wrapper function
  let mut source = String::default();
  source.push_str(&format\!(
    "var __webpack_exec__ = function(moduleId) {{ return __webpack_require__({} = moduleId) }}\n",
    RuntimeGlobals::ENTRY_MODULE_ID
  ));

  // Build comma-separated module execution calls
  let module_ids_code = &module_id_exprs
    .iter()
    .map(|module_id_expr| format\!("__webpack_exec__({module_id_expr})"))
    .collect::<Vec<_>>()
    .join(", ");
  
  // Phase 4: Generate startup code
  if chunks_ids.is_empty() {
    // Simple path: no dependent chunks
    if \!module_ids_code.is_empty() {
      source.push_str("var __webpack_exports__ = (");
      source.push_str(module_ids_code);
      source.push_str(");\n");
    }
  } else {
    // Complex path: has dependent chunks
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

  RawStringSource::from(source).boxed()
}
```

## Call Site 1: ArrayPushCallbackChunkFormatPlugin (array_push_callback_chunk_format.rs:140-189)

```rust
#[plugin_hook(JavascriptModulesRenderChunk for ArrayPushCallbackChunkFormatPlugin)]
async fn render_chunk(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks(compilation.id());
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let has_runtime_modules = compilation
    .chunk_graph
    .has_chunk_runtime_modules(chunk_ukey);
  let global_object = &compilation.options.output.global_object;
  let hot_update_global = &compilation.options.output.hot_update_global;
  let mut source = ConcatSource::default();

  if matches\!(chunk.kind(), ChunkKind::HotUpdate) {
    // ... hot update handling ...
  } else {
    let chunk_loading_global = &compilation.options.output.chunk_loading_global;

    source.add(RawStringSource::from(format\!(
      r#"({}["{}"] = {}["{}"] || []).push([[{}], "#,
      global_object,
      chunk_loading_global,
      global_object,
      chunk_loading_global,
      serde_json::to_string(chunk.expect_id(&compilation.chunk_ids_artifact))
        .expect("json stringify failed"),
    )));
    source.add(render_source.source.clone());
    let has_entry = chunk.has_entry_module(&compilation.chunk_graph);
    if has_entry || has_runtime_modules {
      source.add(RawStringSource::from_static(","));
      source.add(RawStringSource::from(format\!(
        "function({}) {{\n",
        RuntimeGlobals::REQUIRE
      )));
      if has_runtime_modules {
        source.add(render_runtime_modules(compilation, chunk_ukey).await?);
      }
      if has_entry {
        // KEY: THIS IS WHERE generate_entry_startup IS CALLED
        let entries = compilation
          .chunk_graph
          .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);
        let passive = \!compilation.options.experiments.mf_async_startup;
        let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, passive);
        let last_entry_module = entries
          .keys()
          .next_back()
          .expect("should have last entry module");
        let mut render_source = RenderSource {
          source: start_up_source,
        };
        // Call render_startup hook for extensions
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
        source.add(render_source.source);
        let runtime_requirements =
          ChunkGraph::get_tree_runtime_requirements(compilation, chunk_ukey);
        if runtime_requirements.contains(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME) {
          source.add(RawStringSource::from_static(
            "return __webpack_exports__;\n",
          ));
        }
      }
      source.add(RawStringSource::from_static("\n}\n"));
    }
    source.add(RawStringSource::from_static("])"));
  }
  render_source.source = source.boxed();
  Ok(())
}
```

## Call Site 2: CommonJsChunkFormatPlugin (common_js_chunk_format.rs:135-182)

```rust
#[plugin_hook(JavascriptModulesRenderChunk for CommonJsChunkFormatPlugin)]
async fn render_chunk(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks(compilation.id());
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let base_chunk_output_name = get_chunk_output_name(chunk, compilation).await?;
  let mut sources = ConcatSource::default();
  
  sources.add(RawStringSource::from(format\!(
    "exports.ids = [{}];\n",
    json_stringify(chunk.expect_id(&compilation.chunk_ids_artifact))
  )));
  sources.add(RawStringSource::from_static("exports.modules = "));
  sources.add(render_source.source.clone());
  sources.add(RawStringSource::from_static(";\n"));
  
  if compilation
    .chunk_graph
    .has_chunk_runtime_modules(chunk_ukey)
  {
    sources.add(RawStringSource::from_static("exports.runtime = "));
    sources.add(render_chunk_runtime_modules(compilation, chunk_ukey).await?);
    sources.add(RawStringSource::from_static(";\n"));
  }

  if chunk.has_entry_module(&compilation.chunk_graph) {
    let runtime_chunk_output_name = get_runtime_chunk_output_name(compilation, chunk_ukey).await?;
    sources.add(RawStringSource::from(format\!(
      "// load runtime\nvar {} = require({});\n",
      RuntimeGlobals::REQUIRE,
      json_stringify(&get_relative_path(
        base_chunk_output_name
          .trim_start_matches("/")
          .trim_start_matches("\\"),
        &runtime_chunk_output_name
      ))
    )));
    sources.add(RawStringSource::from(format\!(
      "{}(exports)\n",
      RuntimeGlobals::EXTERNAL_INSTALL_CHUNK,
    )));

    // KEY: THIS IS WHERE generate_entry_startup IS CALLED
    let entries = compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);
    let start_up_source = generate_entry_startup(compilation, chunk_ukey, entries, false);
    let last_entry_module = entries
      .keys()
      .next_back()
      .expect("should have last entry module");
    let mut startup_render_source = RenderSource {
      source: start_up_source,
    };
    // Call render_startup hook for extensions
    hooks
      .try_read()
      .expect("should have js plugin drive")
      .render_startup
      .call(
        compilation,
        chunk_ukey,
        last_entry_module,
        &mut startup_render_source,
      )
      .await?;
    sources.add(startup_render_source.source);
    render_source.source = ConcatSource::new([
      RawStringSource::from_static("(function() {\n").boxed(),
      sources.boxed(),
      RawStringSource::from_static("\n})()").boxed(),
    ])
    .boxed();
    return Ok(());
  }
  render_source.source = sources.boxed();
  Ok(())
}
```

## Hook Definition (drive.rs:10-52)

```rust
// Hook definitions
define_hook\!(JavascriptModulesRenderChunk: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, source: &mut RenderSource));
define_hook\!(JavascriptModulesRenderChunkContent: SeriesBail(compilation: &Compilation, chunk_ukey: &ChunkUkey, asset_info: &mut AssetInfo) -> RenderSource);
define_hook\!(JavascriptModulesRender: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, source: &mut RenderSource));
define_hook\!(JavascriptModulesRenderStartup: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey, module: &ModuleIdentifier, source: &mut RenderSource));
define_hook\!(JavascriptModulesRenderModuleContent: Series(compilation: &Compilation, chunk_ukey: &ChunkUkey,module: &dyn Module, source: &mut RenderSource, init_fragments: &mut ChunkInitFragments),tracing=false);
// ... more hooks ...

#[derive(Debug, Default)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct JavascriptModulesPluginHooks {
  #[cfg_attr(allocative, allocative(skip))]
  pub render_chunk: JavascriptModulesRenderChunkHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub render_chunk_content: JavascriptModulesRenderChunkContentHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub render: JavascriptModulesRenderHook,
  #[cfg_attr(allocative, allocative(skip))]
  pub render_startup: JavascriptModulesRenderStartupHook,
  // ... more hooks ...
}

#[derive(Debug)]
pub struct RenderSource {
  pub source: BoxSource,
}
```

## Example Hook Implementation: AssignLibraryPlugin (assign_library_plugin.rs:245-280)

```rust
#[plugin_hook(JavascriptModulesRenderStartup for AssignLibraryPlugin)]
async fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  let mut source = ConcatSource::default();
  source.add(render_source.source.clone());
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let full_name_resolved = self
    .get_resolved_full_name(&options, compilation, chunk)
    .await?;
  let export_access = options
    .export
    .map(|e| property_access(e, 0))
    .unwrap_or_default();
  let full_name_str = if options.name.is_some() {
    format\!(
      "{}{}",
      full_name_resolved.iter().map(|s| property_access(s, 0)).collect::<Vec<_>>().join(""),
      export_access
    )
  } else {
    format\!(
      "{}{}",
      full_name_resolved.iter().map(|s| property_access(s, 0)).collect::<Vec<_>>().join(""),
      export_access
    )
  };
  // Append assignment to exports
  source.add(RawStringSource::from(format\!(
    "{} = __webpack_exports__;\n",
    full_name_str
  )));
  render_source.source = source.boxed();
  Ok(())
}
```

## stringify_chunks_to_array Helper (runtime.rs:370-380)

```rust
pub fn stringify_chunks_to_array(chunks: &HashSet<ChunkId>) -> String {
  let mut v = Vec::from_iter(chunks.iter());
  v.sort_unstable();

  format\!(
    r#"[{}]"#,
    v.iter().fold(String::new(), |prev, cur| {
      prev + format\!(r#""{cur}","#).as_str()
    })
  )
}
```

## Runtime Globals (runtime_globals.rs:10-265)

```rust
bitflags\! {
  impl RuntimeGlobals: u128 {
    // ... many constants ...
    
    /**
     * register deferred code, which will run when certain
     * chunks are loaded.
     * Signature: (chunkIds: Id[], fn: () => any, priority: int >= 0 = 0) => any
     * Returned value will be returned directly when all chunks are already loaded
     * When (priority & 1) it will wait for all other handlers with lower priority to
     * be executed before itself is executed
     */
    const ON_CHUNKS_LOADED = 1 << 15;

    // ... more constants ...

    const STARTUP_ENTRYPOINT = 1 << 34;

    // ... more constants ...

    const ENTRY_MODULE_ID = 1 << 39;

    // ... more constants ...
  }
}

impl RuntimeGlobals {
  pub fn name(&self) -> &'static str {
    match *self {
      R::ON_CHUNKS_LOADED => "__webpack_require__.O",
      R::STARTUP_ENTRYPOINT => "__webpack_require__.X",
      R::ENTRY_MODULE_ID => "__webpack_require__.s",
      // ... more mappings ...
    }
  }
}
```

## Generated Output Examples

### Example 1: Simple Entry (No Dependencies)
```javascript
var __webpack_exec__ = function(moduleId) { 
  return __webpack_require__(__webpack_require__.s = moduleId) 
}
var __webpack_exports__ = (__webpack_exec__(0));
```

### Example 2: Entry with Dependencies (passive=true, ON_CHUNKS_LOADED)
```javascript
var __webpack_exec__ = function(moduleId) { 
  return __webpack_require__(__webpack_require__.s = moduleId) 
}
__webpack_require__.O(0, [1, 2, 3], function() {
  return __webpack_exec__(0);
});
var __webpack_exports__ = __webpack_require__.O();
```

### Example 3: Entry with Dependencies (passive=false, STARTUP_ENTRYPOINT)
```javascript
var __webpack_exec__ = function(moduleId) { 
  return __webpack_require__(__webpack_require__.s = moduleId) 
}
var __webpack_exports__ = __webpack_require__.X(0, [1, 2, 3], function() {
  return __webpack_exec__(0);
});
```

### Example 4: With Library Plugin Extension
```javascript
var __webpack_exec__ = function(moduleId) { 
  return __webpack_require__(__webpack_require__.s = moduleId) 
}
var __webpack_exports__ = (__webpack_exec__(0));
window.myLibrary = __webpack_exports__;
```

