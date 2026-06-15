use rspack_core::{
  ChunkCodeTemplate, ChunkKind, ChunkUkey, Compilation, RuntimeGlobals, RuntimeProxyMetadata,
  RuntimeVariable, property_access,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, SourceExt},
  runtime_mode::RuntimeMode,
};
use rspack_error::Result;

use crate::runtime::render_runtime_module_sources;

fn get_runtime_context_metadata<'a>(
  compilation: &'a Compilation,
  chunk_ukey: &ChunkUkey,
) -> Option<&'a RuntimeProxyMetadata> {
  if compilation.options.experiments.runtime_mode != RuntimeMode::Rspack {
    return None;
  }

  if let Some(metadata) = compilation.runtime_proxy_metadata_artifact.get(chunk_ukey) {
    return Some(metadata);
  }

  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  compilation
    .runtime_proxy_metadata_artifact
    .iter()
    .find_map(|(runtime_chunk_ukey, metadata)| {
      let runtime_chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(runtime_chunk_ukey);
      runtime_chunk
        .runtime()
        .iter()
        .any(|runtime| chunk.runtime().contains(runtime))
        .then_some(metadata)
    })
}

fn runtime_context_current_chunk_metadata(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Option<RuntimeProxyMetadata> {
  if compilation.options.experiments.runtime_mode != RuntimeMode::Rspack {
    return None;
  }

  let mut metadata = compilation
    .runtime_proxy_metadata_artifact
    .get(chunk_ukey)
    .cloned()
    .unwrap_or_default();
  for runtime_module_id in compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_runtime_modules_iterable(chunk_ukey)
  {
    let runtime_module = compilation
      .runtime_modules
      .get(runtime_module_id)
      .expect("should have runtime module");
    let module_runtime_requirements = runtime_module.additional_runtime_requirements(compilation)
      | runtime_module.additional_write_runtime_requirements(compilation);
    metadata
      .tree_runtime_requirements
      .insert(module_runtime_requirements);
    metadata
      .runtime_module_requirements
      .insert(module_runtime_requirements);
  }

  (!metadata.tree_runtime_requirements.is_empty()).then_some(metadata)
}

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

pub async fn render_rspack_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
) -> Result<BoxSource> {
  let runtime_module_sources =
    render_runtime_module_sources(compilation, chunk_ukey, runtime_template, true).await?;
  let mut sources = ConcatSource::default();
  let mut metadata = runtime_context_current_chunk_metadata(compilation, chunk_ukey);
  let script_nonce = RuntimeGlobals::SCRIPT_NONCE
    .to_lexical_name()
    .expect("script nonce should have lexical name");
  if runtime_module_sources
    .iter()
    .any(|(source, _)| source.source().into_string_lossy().contains(script_nonce))
  {
    metadata
      .get_or_insert_default()
      .tree_runtime_requirements
      .insert(RuntimeGlobals::SCRIPT_NONCE);
  }
  let base_metadata = get_runtime_context_metadata(compilation, chunk_ukey);
  let should_render_context = base_metadata.is_some_and(|metadata| {
    metadata
      .tree_runtime_requirements
      .contains(RuntimeGlobals::REQUIRE_SCOPE)
  });
  let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  let owns_runtime = chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);
  let is_hot_update = matches!(chunk.kind(), ChunkKind::HotUpdate);
  let runtime_requirements = compilation
    .cgc_runtime_requirements_artifact
    .get(chunk_ukey)
    .copied()
    .unwrap_or_default();
  let has_bootstrap_runtime_context = runtime_template.uses_runtime_context()
    && owns_runtime
    && runtime_requirements.needs_bootstrap_runtime_context();
  let has_runtime_context = is_hot_update
    || has_bootstrap_runtime_context
    || (!owns_runtime && runtime_template.uses_runtime_context());
  let mut current_chunk_generated_requirements = RuntimeGlobals::default();
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
    current_chunk_generated_requirements
      .insert(runtime_module.additional_write_runtime_requirements(compilation));
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
  if is_hot_update && let Some(metadata) = &mut metadata {
    metadata.tree_runtime_requirements.insert(
      RuntimeGlobals::PUBLIC_PATH
        | RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME
        | RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME
        | RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME
        | RuntimeGlobals::GET_FULL_HASH
        | RuntimeGlobals::LOAD_SCRIPT
        | RuntimeGlobals::HAS_OWN_PROPERTY
        | RuntimeGlobals::MODULE_CACHE
        | RuntimeGlobals::CREATE_SCRIPT_URL,
    );
  }
  let isolate_runtime_context = runtime_template.uses_runtime_context()
    && runtime_template.uses_lexical_runtime_globals()
    && !compilation.options.output.module
    && should_render_context
    && base_metadata.is_some_and(|metadata| {
      !metadata.lexical_fields().is_empty() || !metadata.context_fields().is_empty()
    });

  if should_render_context {
    if isolate_runtime_context {
      sources.add(RawStringSource::from(format!(
        "(function({runtime_context}) {{\n"
      )));
    } else if !has_runtime_context {
      sources.add(RawStringSource::from(render_runtime_context_declaration(
        runtime_template,
      )));
    }

    if let Some(metadata) = &metadata {
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
      let render_runtime_global = |runtime_global: RuntimeGlobals| {
        if is_hot_update
          && runtime_global.intersects(
            RuntimeGlobals::HMR_DOWNLOAD_MANIFEST
              | RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS
              | RuntimeGlobals::HMR_INVALIDATE_MODULE_HANDLERS
              | RuntimeGlobals::HMR_MODULE_DATA,
          )
        {
          return None;
        }
        if is_hot_update {
          return render_context_field(runtime_global);
        }
        let should_render_context_field = owns_runtime
          && runtime_global
            .intersects(RuntimeGlobals::STARTUP | RuntimeGlobals::STARTUP_ENTRYPOINT)
          && metadata
            .bootstrap_proxy_requirements
            .contains(runtime_global)
          || !owns_runtime
            && (runtime_global.needs_bootstrap_runtime_context()
              || metadata
                .runtime_module_requirements
                .contains(runtime_global)
                && !current_chunk_generated_requirements.contains(runtime_global));
        if runtime_global == RuntimeGlobals::REQUIRE {
          Some(runtime_template.render_runtime_variable(&RuntimeVariable::Require))
        } else if runtime_global == RuntimeGlobals::MODULE_FACTORIES
          || runtime_global == RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY
        {
          Some(runtime_template.render_runtime_variable(&RuntimeVariable::Modules))
        } else if runtime_global == RuntimeGlobals::MODULE_CACHE {
          Some(runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache))
        } else if should_render_context_field {
          render_context_field(runtime_global)
        } else {
          None
        }
      };
      sources.add(RawStringSource::from(
        metadata.render_lexical_declarations(Some(&render_runtime_global)),
      ));
      if metadata.lexical_fields().intersects(
        RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS
          | RuntimeGlobals::HMR_INVALIDATE_MODULE_HANDLERS
          | RuntimeGlobals::HMR_MODULE_DATA
          | RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX,
      ) {
        for key in &hmr_state_keys {
          sources.add(RawStringSource::from(format!("var hmrS_{key};\n")));
        }
      }
    }
  }

  if !has_runtime_context
    && runtime_template.uses_lexical_runtime_globals()
    && let Some(metadata) = &metadata
  {
    for (_, runtime_global) in metadata.lexical_fields().iter_names() {
      let Some(key) = runtime_global.rspack_context_property_name() else {
        continue;
      };
      let Some(lexical_name) = runtime_global.to_lexical_name() else {
        continue;
      };
      sources.add(RawStringSource::from(format!(
        "{lexical_name} = {runtime_context}{};\n",
        property_access([key], 0)
      )));
      if runtime_global.should_initialize_as_object() {
        let value = if is_hot_update {
          format!("{lexical_name} || {{}}")
        } else {
          "{}".to_string()
        };
        sources.add(RawStringSource::from(format!(
          "{lexical_name} = {value};\n"
        )));
      } else if runtime_global.should_initialize_as_array() {
        let value = if is_hot_update {
          format!("{lexical_name} || []")
        } else {
          "[]".to_string()
        };
        sources.add(RawStringSource::from(format!(
          "{lexical_name} = {value};\n"
        )));
      }
    }
  }
  if is_hot_update && runtime_template.uses_runtime_context() {
    let require = runtime_template.render_runtime_variable(&RuntimeVariable::Require);
    let modules = runtime_template.render_runtime_variable(&RuntimeVariable::Modules);
    let module_cache = runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache);
    sources.add(RawStringSource::from(format!(
      "var {require}={runtime_context}.r,{modules}={runtime_context}.m,{module_cache}={runtime_context}.c;\n"
    )));
  }
  for (runtime_module_source, generated_requirements) in runtime_module_sources {
    sources.add(runtime_module_source);
    if !runtime_template.uses_lexical_runtime_globals() {
      continue;
    }
    let Some(metadata) = &metadata else {
      continue;
    };
    let mut context_fields = metadata
      .context_fields()
      .intersection(generated_requirements);
    context_fields.insert(generated_requirements.intersection(metadata.tree_runtime_requirements));
    let hmr_live_binding_fields = RuntimeGlobals::GET_FULL_HASH
      | RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME
      | RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME
      | RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME
      | RuntimeGlobals::PUBLIC_PATH;
    if !is_hot_update
      && metadata
        .tree_runtime_requirements
        .contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST)
    {
      context_fields.insert(generated_requirements.intersection(hmr_live_binding_fields));
    }
    if is_hot_update {
      context_fields.insert(generated_requirements);
      context_fields.remove(RuntimeGlobals::REQUIRE | RuntimeGlobals::REQUIRE_SCOPE);
    }
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
      let needs_live_binding = !is_hot_update
        && (metadata.context_setter_fields().contains(runtime_global)
          || hmr_live_binding_fields.contains(runtime_global)
            && metadata
              .tree_runtime_requirements
              .contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST));
      if needs_live_binding {
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
  if isolate_runtime_context {
    sources.add(RawStringSource::from(format!(
      "\n}}).call(this, {runtime_context});\n"
    )));
  }

  Ok(sources.boxed())
}
