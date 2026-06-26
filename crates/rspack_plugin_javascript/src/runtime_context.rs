use std::sync::LazyLock;

use rspack_core::{
  ChunkCodeTemplate, ChunkKind, ChunkUkey, Compilation, RuntimeGlobals, RuntimeProxyMetadata,
  RuntimeVariable, SourceType, property_access,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt},
};
use rspack_error::Result;

use crate::runtime::render_runtime_module_sources;

static HMR_RUNTIME_STATE_GLOBALS: LazyLock<RuntimeGlobals> = LazyLock::new(|| {
  RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS
    | RuntimeGlobals::HMR_INVALIDATE_MODULE_HANDLERS
    | RuntimeGlobals::HMR_MODULE_DATA
    | RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX
});

static LIVE_BINDING_CONTEXT_GLOBALS: LazyLock<RuntimeGlobals> = LazyLock::new(|| {
  RuntimeGlobals::PUBLIC_PATH
    | RuntimeGlobals::SCRIPT_NONCE
    | RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME
    | RuntimeGlobals::SHARE_SCOPE_MAP
    | RuntimeGlobals::INITIALIZE_SHARING
    | RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE
});

pub fn render_runtime_context_declaration(runtime_template: &ChunkCodeTemplate) -> String {
  let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
  format!("var {runtime_context}={{}};\n")
}

pub fn render_runtime_context_require_assignment(runtime_template: &ChunkCodeTemplate) -> String {
  format!(
    "{} = {};\n",
    runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
    runtime_template.render_runtime_variable(&RuntimeVariable::Require)
  )
}

pub async fn render_runtime_chunk_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
) -> Result<BoxSource> {
  let runtime_module_sources =
    render_runtime_module_sources(compilation, chunk_ukey, runtime_template, true, true).await?;
  let mut sources = ConcatSource::default();
  if runtime_module_sources.is_empty() {
    return Ok(sources.boxed());
  }
  let metadata = compilation
    .runtime_proxy_metadata_artifact
    .get(chunk_ukey)
    .expect("should generate runtime metadata");
  let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
  let module_graph = compilation.get_module_graph();
  let has_modules = !compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_modules_identifier_by_source_type(chunk_ukey, SourceType::JavaScript, module_graph)
    .is_empty();
  let is_hmr_runtime = metadata
    .tree_runtime_requirements
    .contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);
  let mut hmr_state_keys = Vec::new();
  for runtime_module_id in compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_runtime_modules_iterable(chunk_ukey)
  {
    let runtime_module = compilation
      .runtime_modules
      .get(runtime_module_id)
      .expect("should have runtime module");
    let Some(key) = (match runtime_module.get_constructor_name().as_str() {
      "JsonpChunkLoadingRuntimeModule" => Some("jsonp"),
      "ModuleChunkLoadingRuntimeModule" => Some("module"),
      "ImportScriptsChunkLoadingRuntimeModule" => Some("importScripts"),
      "ReadFileChunkLoadingRuntimeModule" => Some("readFileVm"),
      "RequireChunkLoadingRuntimeModule" => Some("require"),
      _ => None,
    }) else {
      continue;
    };
    hmr_state_keys.push(key);
  }

  let isolate = has_modules && !compilation.options.output.module;
  if isolate {
    sources.add(RawStringSource::from("(function() {\n".to_string()));
  }

  let render_runtime_global = |runtime_global: RuntimeGlobals| {
    if runtime_global == RuntimeGlobals::REQUIRE {
      Some(runtime_template.render_runtime_variable(&RuntimeVariable::Require))
    } else if runtime_global == RuntimeGlobals::MODULE_FACTORIES
      || runtime_global == RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY
    {
      let modules = runtime_template.render_runtime_variable(&RuntimeVariable::Modules);
      Some(format!(
        "typeof {modules} !== \"undefined\" ? {modules} : {{}}"
      ))
    } else if runtime_global == RuntimeGlobals::MODULE_CACHE {
      let module_cache = runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache);
      Some(format!(
        "typeof {module_cache} !== \"undefined\" ? {module_cache} : {{}}"
      ))
    } else if runtime_global
      .intersects(RuntimeGlobals::STARTUP | RuntimeGlobals::STARTUP_ENTRYPOINT)
    {
      runtime_global
        .rspack_context_property_name()
        .map(|property_name| format!("{runtime_context}{}", property_access([property_name], 0)))
    } else {
      None
    }
  };
  sources.add(RawStringSource::from(
    metadata.render_lexical_declarations(Some(&render_runtime_global)),
  ));
  if metadata
    .lexical_fields()
    .intersects(*HMR_RUNTIME_STATE_GLOBALS)
  {
    for key in &hmr_state_keys {
      sources.add(RawStringSource::from(format!("var hmrS_{key};\n")));
    }
  }
  for (runtime_module_source, generated_requirements, context_requirements) in
    runtime_module_sources
  {
    sources.add(runtime_module_source);
    let mut context_fields = metadata.context_fields().intersection(context_requirements);
    if is_hmr_runtime {
      context_fields.insert(generated_requirements.renderable_require_scope());
      context_fields.remove(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE);
    }
    let setters = metadata.context_setter_fields();
    if context_fields.is_empty() {
      continue;
    }
    for (_, runtime_global) in context_fields.iter_names() {
      let (Some(key), Some(lexical_name)) = (
        runtime_global.rspack_context_property_name(),
        runtime_global.to_lexical_name(),
      ) else {
        continue;
      };
      if setters.contains(runtime_global)
        && (is_hmr_runtime || LIVE_BINDING_CONTEXT_GLOBALS.contains(runtime_global))
      {
        sources.add(RawStringSource::from(format!(
          "Object.defineProperty({}, {}, {{ configurable: true, get: function() {{ return {}; }}, set: function(value) {{ {} = value; }} }});\n",
          runtime_context,
          rspack_util::json_stringify(key),
          lexical_name,
          lexical_name
        )));
      } else {
        sources.add(RawStringSource::from(format!(
          "{}{} = {};\n",
          runtime_context,
          property_access([key], 0),
          lexical_name
        )));
      }
    }
  }
  if isolate {
    sources.add(RawStringSource::from("\n}).call(this);\n".to_string()));
  }

  Ok(sources.boxed())
}

pub async fn render_chunk_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
) -> Result<BoxSource> {
  let runtime_module_sources =
    render_runtime_module_sources(compilation, chunk_ukey, runtime_template, true, true).await?;
  let mut sources = ConcatSource::default();
  if runtime_module_sources.is_empty() {
    return Ok(sources.boxed());
  }
  let metadata = compilation
    .runtime_proxy_metadata_artifact
    .get(chunk_ukey)
    .expect("should generate runtime metadata");
  let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
  let is_hmr_runtime = metadata
    .tree_runtime_requirements
    .contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);

  sources.add(RawStringSource::from("(function() {\n".to_string()));
  let render_context_field = |runtime_global: RuntimeGlobals| {
    runtime_global
      .rspack_context_property_name()
      .map(|property_name| {
        let value = format!("{runtime_context}{}", property_access([property_name], 0));
        if runtime_global.should_initialize_as_object() {
          format!("{value}||{{}}")
        } else if runtime_global.should_initialize_as_array() {
          format!("{value}||[]")
        } else {
          value
        }
      })
  };
  let render_runtime_global = |runtime_global: RuntimeGlobals| render_context_field(runtime_global);
  sources.add(RawStringSource::from(
    metadata.render_lexical_declarations(Some(&render_runtime_global)),
  ));

  for (runtime_module_source, generated_requirements, context_requirements) in
    runtime_module_sources
  {
    sources.add(runtime_module_source);
    let mut context_fields = metadata.context_fields().intersection(context_requirements);
    if is_hmr_runtime {
      context_fields.insert(generated_requirements.renderable_require_scope());
      context_fields.remove(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE);
    }

    let setters = metadata.context_setter_fields();
    for (_, runtime_global) in context_fields.iter_names() {
      let (Some(key), Some(lexical_name)) = (
        runtime_global.rspack_context_property_name(),
        runtime_global.to_lexical_name(),
      ) else {
        continue;
      };
      if setters.contains(runtime_global)
        && (is_hmr_runtime || LIVE_BINDING_CONTEXT_GLOBALS.contains(runtime_global))
      {
        sources.add(RawStringSource::from(format!(
          "Object.defineProperty({}, {}, {{ configurable: true, get: function() {{ return {}; }}, set: function(value) {{ {} = value; }} }});\n",
          runtime_context,
          rspack_util::json_stringify(key),
          lexical_name,
          lexical_name
        )));
      } else {
        sources.add(RawStringSource::from(format!(
          "{}{} = {};\n",
          runtime_context,
          property_access([key], 0),
          lexical_name
        )));
      }
    }
  }

  sources.add(RawStringSource::from("\n}).call(this);\n".to_string()));

  Ok(sources.boxed())
}

pub async fn render_hot_update_chunk_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
) -> Result<BoxSource> {
  let runtime_module_sources =
    render_runtime_module_sources(compilation, chunk_ukey, runtime_template, true, false).await?;
  let mut sources = ConcatSource::default();
  if runtime_module_sources.is_empty() {
    return Ok(sources.boxed());
  }
  let metadata = runtime_context_current_chunk_metadata(compilation, chunk_ukey);

  let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
  let mut hmr_state_keys = Vec::new();
  for runtime_module_id in compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_runtime_modules_iterable(chunk_ukey)
  {
    let runtime_module = compilation
      .runtime_modules
      .get(runtime_module_id)
      .expect("should have runtime module");
    let Some(key) = (match runtime_module.get_constructor_name().as_str() {
      "JsonpChunkLoadingRuntimeModule" => Some("jsonp"),
      "ModuleChunkLoadingRuntimeModule" => Some("module"),
      "ImportScriptsChunkLoadingRuntimeModule" => Some("importScripts"),
      "ReadFileChunkLoadingRuntimeModule" => Some("readFileVm"),
      "RequireChunkLoadingRuntimeModule" => Some("require"),
      _ => None,
    }) else {
      continue;
    };
    hmr_state_keys.push(key);
  }

  let render_context_field = |runtime_global: RuntimeGlobals| {
    runtime_global
      .rspack_context_property_name()
      .map(|property_name| {
        let value = format!("{runtime_context}{}", property_access([property_name], 0));
        if runtime_global.should_initialize_as_object() {
          format!("{value}||{{}}")
        } else if runtime_global.should_initialize_as_array() {
          format!("{value}||[]")
        } else {
          value
        }
      })
  };
  sources.add(RawStringSource::from(
    metadata.render_lexical_declarations(Some(&render_context_field)),
  ));
  if metadata
    .lexical_fields()
    .intersects(*HMR_RUNTIME_STATE_GLOBALS)
  {
    for key in &hmr_state_keys {
      sources.add(RawStringSource::from(format!("var hmrS_{key};\n")));
    }
  }

  let require = runtime_template.render_runtime_variable(&RuntimeVariable::Require);
  let modules = runtime_template.render_runtime_variable(&RuntimeVariable::Modules);
  let module_cache = runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache);
  sources.add(RawStringSource::from(format!(
    "var {require}={runtime_context}.r,{modules}={runtime_context}.m,{module_cache}={runtime_context}.c;\n"
  )));

  for (runtime_module_source, generated_requirements, _) in runtime_module_sources {
    sources.add(runtime_module_source);
    let mut context_fields = metadata
      .context_fields()
      .intersection(generated_requirements);
    context_fields.insert(generated_requirements.renderable_require_scope());
    context_fields.remove(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE);
    if context_fields.is_empty() {
      continue;
    }
    for (_, runtime_global) in context_fields.iter_names() {
      let (Some(key), Some(lexical_name)) = (
        runtime_global.rspack_context_property_name(),
        runtime_global.to_lexical_name(),
      ) else {
        continue;
      };
      sources.add(RawStringSource::from(format!(
        ";{}{} = {};\n",
        runtime_context,
        property_access([key], 0),
        lexical_name
      )));
    }
  }

  Ok(sources.boxed())
}

pub async fn render_rspack_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
) -> Result<BoxSource> {
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  if matches!(chunk.kind(), ChunkKind::HotUpdate) {
    render_hot_update_chunk_runtime_modules(compilation, chunk_ukey, runtime_template).await
  } else if chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
    render_runtime_chunk_runtime_modules(compilation, chunk_ukey, runtime_template).await
  } else {
    render_chunk_runtime_modules(compilation, chunk_ukey, runtime_template).await
  }
}

fn runtime_context_current_chunk_metadata(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> RuntimeProxyMetadata {
  let mut metadata = compilation
    .runtime_proxy_metadata_artifact
    .get(chunk_ukey)
    .cloned()
    .unwrap_or_default();
  if let Some(chunk_runtime_requirements) = compilation
    .cgc_runtime_requirements_artifact
    .get(chunk_ukey)
  {
    metadata
      .tree_runtime_requirements
      .insert(*chunk_runtime_requirements);
    metadata
      .runtime_module_requirements
      .insert(*chunk_runtime_requirements);
  }
  for runtime_module_id in compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_runtime_modules_iterable(chunk_ukey)
  {
    let runtime_module = compilation
      .runtime_modules
      .get(runtime_module_id)
      .expect("should have runtime module");
    let module_runtime_requirements = runtime_module.runtime_requirements(compilation);
    metadata
      .tree_runtime_requirements
      .insert(module_runtime_requirements.lexical_requirements());
    metadata
      .runtime_module_requirements
      .insert(module_runtime_requirements.dependencies);
    metadata
      .context_setter_fields
      .insert(module_runtime_requirements.write);
    metadata
      .force_context_fields
      .insert(module_runtime_requirements.force_context);
  }

  metadata
}
