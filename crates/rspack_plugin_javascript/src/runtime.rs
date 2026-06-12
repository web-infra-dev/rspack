use rayon::prelude::*;
use rspack_core::{
  ChunkCodeTemplate, ChunkGraph, ChunkInitFragments, ChunkKind, ChunkUkey,
  CodeGenerationPublicPathAutoReplace, Compilation, Module, RuntimeGlobals, RuntimeModule,
  RuntimeModuleGenerateContext, RuntimeProxyMetadata, RuntimeVariable, SourceType,
  chunk_graph_chunk::ChunkIdSet,
  get_undo_path, property_access, render_runtime_module_source,
  rspack_sources::{
    BoxSource, ConcatSource, OriginalSource, RawStringSource, ReplaceSource, Source, SourceExt,
  },
  runtime_mode::RuntimeMode,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};

use crate::{JavascriptModulesPluginHooks, RenderSource};

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_ASSET_AUTO_PUBLIC_PATH__";

pub async fn render_chunk_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  ordered_modules: &Vec<&dyn Module>,
  all_strict: bool,
  output_path: &str,
  hooks: &JavascriptModulesPluginHooks,
  runtime_template: &ChunkCodeTemplate,
) -> Result<Option<(BoxSource, ChunkInitFragments)>> {
  let module_sources = rspack_parallel::scope::<_, _>(|token| {
    ordered_modules.iter().for_each(|module| {
      let s = unsafe {
        token.used((
          compilation,
          chunk_ukey,
          module,
          all_strict,
          output_path,
          hooks,
          runtime_template
        ))
      };
      s.spawn(
        |(compilation, chunk_ukey, module, all_strict, output_path, hooks, runtime_template)| async move {
          render_module(
            compilation,
            chunk_ukey,
            *module,
            all_strict,
            true,
            output_path,
            hooks,
            runtime_template
          )
          .await
          .map(|result| result.map(|(s, f, a)| (module.identifier(), s, f, a)))
        },
      );
    });
  })
  .await
  .into_iter()
  .map(|r| r.to_rspack_result())
  .collect::<Result<Vec<_>>>()?;

  let mut module_code_array = vec![];
  for item in module_sources {
    if let Some(i) = item? {
      module_code_array.push(i);
    }
  }

  if module_code_array.is_empty() {
    return Ok(None);
  }

  module_code_array.sort_unstable_by_key(|(module_identifier, _, _, _)| *module_identifier);

  let chunk_init_fragments = module_code_array.iter().fold(
    ChunkInitFragments::default(),
    |mut chunk_init_fragments, (_, _, fragments, additional_fragments)| {
      chunk_init_fragments.extend((*fragments).clone());
      chunk_init_fragments.extend(additional_fragments.clone());
      chunk_init_fragments
    },
  );

  let module_sources: Vec<_> = module_code_array
    .into_iter()
    .map(|(_, source, _, _)| source)
    .collect();
  let module_sources = module_sources
    .into_par_iter()
    .fold(ConcatSource::default, |mut output, source| {
      output.add(source);
      output
    })
    .collect::<Vec<ConcatSource>>();

  let mut sources = ConcatSource::default();
  sources.add(RawStringSource::from_static("{\n"));
  sources.add(ConcatSource::new(module_sources));
  sources.add(RawStringSource::from_static("\n}"));

  Ok(Some((sources.boxed(), chunk_init_fragments)))
}

#[allow(clippy::too_many_arguments)]
pub async fn render_module(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &dyn Module,
  all_strict: bool,
  factory: bool,
  output_path: &str,
  hooks: &JavascriptModulesPluginHooks,
  runtime_template: &ChunkCodeTemplate,
) -> Result<Option<(BoxSource, ChunkInitFragments, ChunkInitFragments)>> {
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  let code_gen_result = compilation
    .code_generation_results
    .get(&module.identifier(), Some(chunk.runtime()));
  let Some(origin_source) = code_gen_result.get(&SourceType::JavaScript) else {
    return Ok(None);
  };

  let mut module_chunk_init_fragments = match code_gen_result.data.get::<ChunkInitFragments>() {
    Some(fragments) => fragments.clone(),
    None => ChunkInitFragments::default(),
  };

  let mut render_source = if code_gen_result
    .data
    .get::<CodeGenerationPublicPathAutoReplace>()
    .is_some()
  {
    let content = origin_source.source().into_string_lossy();
    let len = AUTO_PUBLIC_PATH_PLACEHOLDER.len();
    let auto_public_path_matches: Vec<_> = content
      .match_indices(AUTO_PUBLIC_PATH_PLACEHOLDER)
      .map(|(index, _)| (index, index + len))
      .collect();
    if !auto_public_path_matches.is_empty() {
      let mut replace = ReplaceSource::new(origin_source.clone());
      for (start, end) in auto_public_path_matches {
        let relative = get_undo_path(
          output_path,
          compilation.options.output.path.to_string(),
          true,
        );
        replace.replace(start as u32, end as u32, relative, None);
      }
      RenderSource {
        source: replace.boxed(),
      }
    } else {
      RenderSource {
        source: origin_source.clone(),
      }
    }
  } else {
    RenderSource {
      source: origin_source.clone(),
    }
  };

  /*
  If supports method shorthand, render function factory as:
  "./module.js"(module) { code }
  Otherwise render as:
  "./module.js": (function(module) { code })
  */
  let use_method_shorthand = compilation
    .options
    .output
    .environment
    .supports_method_shorthand();

  hooks
    .render_module_content
    .call(
      compilation,
      chunk_ukey,
      module,
      &mut render_source,
      &mut module_chunk_init_fragments,
      runtime_template,
    )
    .await?;

  let sources = if factory {
    let mut sources = ConcatSource::default();
    let module_id =
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier())
        .expect("should have module_id in render_module");
    sources.add(RawStringSource::from(rspack_util::json_stringify(
      module_id,
    )));

    let mut post_module_container = {
      let runtime_requirements = ChunkGraph::get_module_runtime_requirements(
        compilation,
        module.identifier(),
        chunk.runtime(),
      );

      let need_module = runtime_requirements.is_some_and(|r| r.contains(RuntimeGlobals::MODULE));
      let need_exports = runtime_requirements.is_some_and(|r| r.contains(RuntimeGlobals::EXPORTS));
      let need_require = runtime_requirements.is_some_and(|r| {
        r.contains(RuntimeGlobals::REQUIRE)
          || r.contains(RuntimeGlobals::REQUIRE_SCOPE)
          || (compilation.options.experiments.runtime_mode == RuntimeMode::Rspack
            && !r.renderable_require_scope().is_empty())
      });

      let mut args = Vec::new();
      if need_module || need_exports || need_require {
        let module_argument = runtime_template.render_module_argument(module.get_module_argument());
        args.push(if need_module {
          module_argument
        } else {
          format!("__unused_rspack_{module_argument}")
        });
      }

      if need_exports || need_require {
        let exports_argument =
          runtime_template.render_exports_argument(module.get_exports_argument());
        args.push(if need_exports {
          exports_argument
        } else {
          format!("__unused_rspack_{exports_argument}")
        });
      }
      if need_require {
        args.push(runtime_template.render_runtime_argument());
      }

      let mut container_sources = ConcatSource::default();

      if use_method_shorthand {
        container_sources.add(RawStringSource::from(format!("({}) {{\n", args.join(", "))));
      } else {
        container_sources.add(RawStringSource::from(format!(
          ": (function ({}) {{\n",
          args.join(", ")
        )));
      }
      if module.build_info().strict && !all_strict {
        container_sources.add(RawStringSource::from_static("\"use strict\";\n"));
      }
      container_sources.add(render_source.source);

      if use_method_shorthand {
        container_sources.add(RawStringSource::from_static("\n\n},\n"));
      } else {
        container_sources.add(RawStringSource::from_static("\n\n}),\n"));
      }

      RenderSource {
        source: container_sources.boxed(),
      }
    };

    hooks
      .render_module_container
      .call(
        compilation,
        chunk_ukey,
        module,
        &mut post_module_container,
        &mut module_chunk_init_fragments,
        runtime_template,
      )
      .await?;

    let mut post_module_package = post_module_container;

    hooks
      .render_module_package
      .call(
        compilation,
        chunk_ukey,
        module,
        &mut post_module_package,
        &mut module_chunk_init_fragments,
        runtime_template,
      )
      .await?;

    sources.add(post_module_package.source);
    sources.boxed()
  } else {
    hooks
      .render_module_package
      .call(
        compilation,
        chunk_ukey,
        module,
        &mut render_source,
        &mut module_chunk_init_fragments,
        runtime_template,
      )
      .await?;

    render_source.source
  };

  Ok(Some((
    sources,
    code_gen_result.chunk_init_fragments.clone(),
    module_chunk_init_fragments,
  )))
}

fn runtime_context_metadata<'a>(
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

fn runtime_context_render_metadata(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Option<RuntimeProxyMetadata> {
  runtime_context_metadata(compilation, chunk_ukey).cloned()
}

fn runtime_context_current_chunk_metadata(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Option<RuntimeProxyMetadata> {
  if compilation.options.experiments.runtime_mode != RuntimeMode::Rspack {
    return None;
  }

  if let Some(metadata) = compilation.runtime_proxy_metadata_artifact.get(chunk_ukey) {
    return Some(metadata.clone());
  }

  let mut metadata = RuntimeProxyMetadata::default();
  for runtime_module_id in compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_runtime_modules_iterable(chunk_ukey)
  {
    let runtime_module = compilation
      .runtime_modules
      .get(runtime_module_id)
      .expect("should have runtime module");
    let additional_runtime_requirements =
      runtime_module.additional_runtime_requirements(compilation);
    let generated_runtime_requirements = runtime_module_generated_requirements(&**runtime_module);
    metadata
      .tree_runtime_requirements
      .insert(additional_runtime_requirements);
    metadata
      .tree_runtime_requirements
      .insert(generated_runtime_requirements);
    metadata
      .runtime_module_requirements
      .insert(additional_runtime_requirements);
  }

  (!metadata.tree_runtime_requirements.is_empty()).then_some(metadata)
}

pub fn should_render_runtime_context(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> bool {
  if compilation.options.experiments.runtime_mode != RuntimeMode::Rspack {
    return false;
  }

  runtime_context_render_metadata(compilation, chunk_ukey).is_some_and(|metadata| {
    metadata
      .tree_runtime_requirements
      .contains(RuntimeGlobals::REQUIRE_SCOPE)
  })
}

fn is_hot_update_chunk(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> bool {
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  matches!(chunk.kind(), ChunkKind::HotUpdate)
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

fn runtime_module_generated_requirements(runtime_module: &dyn RuntimeModule) -> RuntimeGlobals {
  let constructor_name = runtime_module.get_constructor_name();
  let module_name = runtime_module.identifier().to_string();
  match constructor_name.as_str() {
    "AsyncRuntimeModule" => {
      RuntimeGlobals::ASYNC_MODULE
        | RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL
        | RuntimeGlobals::DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES
        | RuntimeGlobals::DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES_SYMBOL
    }
    "BaseUriRuntimeModule" => RuntimeGlobals::BASE_URI,
    "PublicPathRuntimeModule" | "AutoPublicPathRuntimeModule" => RuntimeGlobals::PUBLIC_PATH,
    "GetChunkFilenameRuntimeModule" if module_name.contains("javascript") => {
      RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME
    }
    "GetChunkFilenameRuntimeModule" if module_name.contains("css") => {
      RuntimeGlobals::GET_CHUNK_CSS_FILENAME
    }
    "GetChunkUpdateFilenameRuntimeModule" => RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME,
    "GetMainFilenameRuntimeModule" => RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME,
    "LoadScriptRuntimeModule" => RuntimeGlobals::LOAD_SCRIPT,
    "HasOwnPropertyRuntimeModule" => RuntimeGlobals::HAS_OWN_PROPERTY,
    "GetFullHashRuntimeModule" => RuntimeGlobals::GET_FULL_HASH,
    "GlobalRuntimeModule" => RuntimeGlobals::GLOBAL,
    "CreateScriptUrlRuntimeModule" => RuntimeGlobals::CREATE_SCRIPT_URL,
    "CreateScriptRuntimeModule" => RuntimeGlobals::CREATE_SCRIPT,
    "OnChunkLoadedRuntimeModule" => RuntimeGlobals::ON_CHUNKS_LOADED,
    "DefinePropertyGettersRuntimeModule" => RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
    "GetTrustedTypesPolicyRuntimeModule" => RuntimeGlobals::GET_TRUSTED_TYPES_POLICY,
    "CreateFakeNamespaceObjectRuntimeModule" => RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
    "MakeNamespaceObjectRuntimeModule" => RuntimeGlobals::MAKE_NAMESPACE_OBJECT,
    "CompatGetDefaultExportRuntimeModule" => RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT,
    "ESMModuleDecoratorRuntimeModule" => RuntimeGlobals::ESM_MODULE_DECORATOR,
    "NodeModuleDecoratorRuntimeModule" => RuntimeGlobals::NODE_MODULE_DECORATOR,
    "SystemContextRuntimeModule" => RuntimeGlobals::SYSTEM_CONTEXT,
    "NonceRuntimeModule" => RuntimeGlobals::SCRIPT_NONCE,
    "RelativeUrlRuntimeModule" => RuntimeGlobals::RELATIVE_URL,
    "ChunkNameRuntimeModule" => RuntimeGlobals::CHUNK_NAME,
    "RuntimeIdRuntimeModule" => RuntimeGlobals::RUNTIME_ID,
    "ShareRuntimeModule" => RuntimeGlobals::INITIALIZE_SHARING | RuntimeGlobals::SHARE_SCOPE_MAP,
    "RemoteRuntimeModule" => RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE,
    "HotModuleReplacementRuntimeModule" => RuntimeGlobals::INTERCEPT_MODULE_EXECUTION,
    "EmbedFederationRuntimeModule" => RuntimeGlobals::STARTUP,
    "StartupEntrypointRuntimeModule" => RuntimeGlobals::STARTUP_ENTRYPOINT,
    "StartupChunkDependenciesRuntimeModule" => RuntimeGlobals::STARTUP,
    "EnsureChunkRuntimeModule" => {
      RuntimeGlobals::ENSURE_CHUNK | RuntimeGlobals::ENSURE_CHUNK_HANDLERS
    }
    "JsonpChunkLoadingRuntimeModule"
    | "ModuleChunkLoadingRuntimeModule"
    | "ImportScriptsChunkLoadingRuntimeModule"
    | "ReadFileChunkLoadingRuntimeModule"
    | "RequireChunkLoadingRuntimeModule" => {
      RuntimeGlobals::BASE_URI
        | RuntimeGlobals::ENSURE_CHUNK_HANDLERS
        | RuntimeGlobals::EXTERNAL_INSTALL_CHUNK
        | RuntimeGlobals::HMR_DOWNLOAD_MANIFEST
        | RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS
        | RuntimeGlobals::HMR_INVALIDATE_MODULE_HANDLERS
        | RuntimeGlobals::HMR_MODULE_DATA
    }
    "EsmChunkLoadingRuntimeModule" => RuntimeGlobals::EXTERNAL_INSTALL_CHUNK,
    "ChunkPrefetchPreloadFunctionRuntimeModule" if module_name.contains("prefetch") => {
      RuntimeGlobals::PREFETCH_CHUNK | RuntimeGlobals::PREFETCH_CHUNK_HANDLERS
    }
    "ChunkPrefetchPreloadFunctionRuntimeModule" if module_name.contains("preload") => {
      RuntimeGlobals::PRELOAD_CHUNK | RuntimeGlobals::PRELOAD_CHUNK_HANDLERS
    }
    "AmdDefineRuntimeModule" => RuntimeGlobals::AMD_DEFINE,
    "AmdOptionsRuntimeModule" => RuntimeGlobals::AMD_OPTIONS,
    "MakeDeferredNamespaceObjectRuntimeModule" => RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT,
    "MakeOptimizedDeferredNamespaceObjectRuntimeModule" => {
      RuntimeGlobals::MAKE_OPTIMIZED_DEFERRED_NAMESPACE_OBJECT
    }
    "ToBinaryRuntimeModule" => RuntimeGlobals::TO_BINARY,
    _ => RuntimeGlobals::default(),
  }
}

fn has_bootstrap_runtime_context(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
) -> bool {
  if !runtime_template.uses_runtime_context() {
    return false;
  }

  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  if !chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
    return false;
  }

  let runtime_requirements = compilation
    .cgc_runtime_requirements_artifact
    .get(chunk_ukey)
    .copied()
    .unwrap_or_default();
  runtime_requirements.needs_bootstrap_runtime_context()
}

fn render_runtime_context_declarations(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
  render_lexical_fields: bool,
  render_context_object: bool,
) -> Option<BoxSource> {
  if !should_render_runtime_context(compilation, chunk_ukey) {
    return None;
  }

  let mut sources = ConcatSource::default();
  if render_context_object {
    sources.add(RawStringSource::from(render_runtime_context_declaration(
      runtime_template,
    )));
  }

  if render_lexical_fields
    && let Some(mut metadata) = runtime_context_current_chunk_metadata(compilation, chunk_ukey)
  {
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(chunk_ukey);
    let owns_runtime =
      chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);
    let current_chunk_generated_requirements = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_runtime_modules_iterable(chunk_ukey)
      .fold(
        RuntimeGlobals::default(),
        |mut requirements, runtime_module_id| {
          let runtime_module = compilation
            .runtime_modules
            .get(runtime_module_id)
            .expect("should have runtime module");
          requirements.insert(runtime_module_generated_requirements(&**runtime_module));
          requirements
        },
      );
    let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
    let is_hot_update = is_hot_update_chunk(compilation, chunk_ukey);
    if is_hot_update {
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
    let render_runtime_global = |runtime_global: RuntimeGlobals| {
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
        && runtime_global.intersects(RuntimeGlobals::STARTUP | RuntimeGlobals::STARTUP_ENTRYPOINT)
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
      for runtime_module_id in compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_runtime_modules_iterable(chunk_ukey)
      {
        let runtime_module = compilation
          .runtime_modules
          .get(runtime_module_id)
          .expect("should have runtime module");
        let key = match runtime_module.get_constructor_name().as_str() {
          "JsonpChunkLoadingRuntimeModule" => "jsonp",
          "ModuleChunkLoadingRuntimeModule" => "module",
          "ImportScriptsChunkLoadingRuntimeModule" => "importScripts",
          "ReadFileChunkLoadingRuntimeModule" => "readFileVm",
          "RequireChunkLoadingRuntimeModule" => "require",
          _ => continue,
        };
        sources.add(RawStringSource::from(format!("var hmrS_{key};\n")));
      }
    }
  }
  if !render_context_object {
    return Some(sources.boxed());
  }

  Some(sources.boxed())
}

fn render_runtime_context_exposed_fields(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
  generated_requirements: RuntimeGlobals,
) -> Option<BoxSource> {
  if !runtime_template.uses_lexical_runtime_globals() {
    return None;
  }

  let metadata = runtime_context_metadata(compilation, chunk_ukey)?;
  let is_hot_update = is_hot_update_chunk(compilation, chunk_ukey);
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
    return None;
  }

  let mut sources = ConcatSource::default();
  let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);

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

  Some(sources.boxed())
}

fn render_runtime_context_field_initializers(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
  preserve_initialized_fields: bool,
) -> Option<BoxSource> {
  if !runtime_template.uses_lexical_runtime_globals() {
    return None;
  }

  let metadata = runtime_context_current_chunk_metadata(compilation, chunk_ukey)?;
  let mut sources = ConcatSource::default();
  let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);

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
      let value = if preserve_initialized_fields {
        format!("{lexical_name} || {{}}")
      } else {
        "{}".to_string()
      };
      sources.add(RawStringSource::from(format!(
        "{lexical_name} = {value};\n"
      )));
    } else if runtime_global.should_initialize_as_array() {
      let value = if preserve_initialized_fields {
        format!("{lexical_name} || []")
      } else {
        "[]".to_string()
      };
      sources.add(RawStringSource::from(format!(
        "{lexical_name} = {value};\n"
      )));
    }
  }

  Some(sources.boxed())
}

fn render_hot_update_runtime_variable_bindings(
  runtime_template: &ChunkCodeTemplate,
) -> Option<BoxSource> {
  if !runtime_template.uses_runtime_context() {
    return None;
  }

  let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
  let require = runtime_template.render_runtime_variable(&RuntimeVariable::Require);
  let modules = runtime_template.render_runtime_variable(&RuntimeVariable::Modules);
  let module_cache = runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache);
  Some(
    RawStringSource::from(format!(
      "var {require}={runtime_context}.r,{modules}={runtime_context}.m,{module_cache}={runtime_context}.c;\n"
    ))
    .boxed(),
  )
}

pub async fn render_chunk_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
) -> Result<BoxSource> {
  let runtime_modules_sources =
    render_runtime_modules(compilation, chunk_ukey, runtime_template).await?;
  if runtime_modules_sources.source().is_empty()
    && !should_render_runtime_context(compilation, chunk_ukey)
  {
    return Ok(runtime_modules_sources);
  }

  let mut sources = ConcatSource::default();
  sources.add(RawStringSource::from(format!(
    "function({}) {{\n",
    runtime_template.render_runtime_argument()
  )));
  sources.add(runtime_modules_sources);
  sources.add(RawStringSource::from_static("\n}\n"));
  Ok(sources.boxed())
}

pub async fn render_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
) -> Result<BoxSource> {
  let mut sources = ConcatSource::default();
  let mut runtime_module_sources = rspack_parallel::scope::<_, Result<_>>(|token| {
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_runtime_modules_in_order(chunk_ukey, compilation)
      .enumerate()
      .map(|(index, (identifier, runtime_module))| {
        (
          index,
          runtime_module.stage(),
          compilation
            .runtime_modules_code_generation_source
            .get(identifier)
            .expect("should have runtime module result"),
          runtime_module,
        )
      })
      .for_each(|(index, stage, source, module)| {
        let s = unsafe { token.used((compilation, source, module, runtime_template)) };
        s.spawn(
          move |(compilation, source, module, runtime_template)| async move {
            if source.size() == 0 {
              return Ok((
                index,
                stage,
                ConcatSource::default().boxed(),
                RuntimeGlobals::default(),
              ));
            }
            let generated_requirements = runtime_module_generated_requirements(module);
            if runtime_template.uses_runtime_context()
              && (module.get_custom_source().is_some()
                || module.get_constructor_name() == "RuntimeModuleFromJs")
            {
              return Err(rspack_error::error!(
                "Custom runtime modules are not supported when `experiments.runtimeMode` is \"rspack\" (runtime module: {}).",
                module.identifier()
              ));
            }
            let supports_arrow_function = compilation
              .options
              .output
              .environment
              .supports_arrow_function();
            let source = if !(module.full_hash()
              || module.dependent_hash()
              || (runtime_template.uses_runtime_context()
                && !runtime_template.uses_lexical_runtime_globals()))
            {
              if let Some(custom_source) = module.get_custom_source() {
                RawStringSource::from(custom_source).boxed()
              } else {
                source.clone()
              }
            } else {
              if let Some(custom_source) = module.get_custom_source() {
                RawStringSource::from(custom_source).boxed()
              } else {
                let runtime_template = compilation.runtime_template.create_runtime_code_template();
                let context = RuntimeModuleGenerateContext {
                  compilation,
                  runtime_template: &runtime_template,
                };
                let source_str = module.generate(&context).await?;
                if module.get_source_map_kind().enabled() {
                  OriginalSource::new(source_str, module.identifier().as_str()).boxed()
                } else {
                  RawStringSource::from(source_str).boxed()
                }
              }
            };
            let sources = render_runtime_module_source(
              module.identifier(),
              source,
              module.should_isolate(),
              supports_arrow_function,
            );
            Ok((index, stage, sources, generated_requirements))
          },
        );
      })
  })
  .await
  .into_iter()
  .map(|r| r.to_rspack_result())
  .collect::<Result<Vec<_>>>()?
  .into_iter()
  .collect::<Result<Vec<_>>>()?;
  runtime_module_sources.sort_by(|(a_index, a_stage, _, _), (b_index, b_stage, _, _)| {
    a_stage.cmp(b_stage).then_with(|| a_index.cmp(b_index))
  });

  let isolate_runtime_context = runtime_template.uses_runtime_context()
    && runtime_template.uses_lexical_runtime_globals()
    && !compilation.options.output.module
    && should_render_runtime_context(compilation, chunk_ukey)
    && runtime_context_render_metadata(compilation, chunk_ukey).is_some_and(|metadata| {
      !metadata.lexical_fields().is_empty() || !metadata.context_fields().is_empty()
    });

  let is_hot_update = is_hot_update_chunk(compilation, chunk_ukey);
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  let owns_runtime = chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);
  let has_runtime_context = is_hot_update
    || has_bootstrap_runtime_context(compilation, chunk_ukey, runtime_template)
    || (!owns_runtime && runtime_template.uses_runtime_context());
  if isolate_runtime_context {
    let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
    sources.add(RawStringSource::from(format!(
      "(function({runtime_context}) {{\n"
    )));
    if let Some(context_declarations) =
      render_runtime_context_declarations(compilation, chunk_ukey, runtime_template, true, false)
    {
      sources.add(context_declarations);
    }
  } else if let Some(context_declarations) = render_runtime_context_declarations(
    compilation,
    chunk_ukey,
    runtime_template,
    true,
    !has_runtime_context,
  ) {
    sources.add(context_declarations);
  }
  if !has_runtime_context
    && let Some(context_fields) = render_runtime_context_field_initializers(
      compilation,
      chunk_ukey,
      runtime_template,
      is_hot_update,
    )
  {
    sources.add(context_fields);
  }
  if is_hot_update
    && let Some(bindings) = render_hot_update_runtime_variable_bindings(runtime_template)
  {
    sources.add(bindings);
  }
  for (_, _, runtime_module_source, generated_requirements) in runtime_module_sources {
    let context_fields = render_runtime_context_exposed_fields(
      compilation,
      chunk_ukey,
      runtime_template,
      generated_requirements,
    );
    sources.add(runtime_module_source);
    if let Some(context_fields) = context_fields {
      sources.add(context_fields);
    }
  }
  if isolate_runtime_context {
    let runtime_context = runtime_template.render_runtime_variable(&RuntimeVariable::Context);
    sources.add(RawStringSource::from(format!(
      "\n}})({runtime_context});\n"
    )));
  }

  Ok(sources.boxed())
}

pub fn stringify_chunks_to_array(chunks: &ChunkIdSet) -> String {
  let mut v = chunks.iter().collect::<Vec<_>>();
  v.sort_unstable();
  rspack_util::json_stringify(&v)
}

pub fn stringify_array(vec: &[String]) -> String {
  format!(
    r#"[{}]"#,
    vec
      .iter()
      .map(|item| format!("\"{item}\""))
      .collect::<Vec<_>>()
      .join(", ")
  )
}

#[cfg(test)]
mod tests {
  use rspack_core::chunk_graph_chunk::ChunkIdSet;

  use super::stringify_chunks_to_array;

  #[test]
  fn stringify_chunks_to_array_uses_chunk_id_serialize() {
    let chunks = ChunkIdSet::from_iter([
      rspack_core::chunk_graph_chunk::ChunkId::from("681"),
      rspack_core::chunk_graph_chunk::ChunkId::from("main"),
    ]);

    assert_eq!(stringify_chunks_to_array(&chunks), "[681,\"main\"]");
  }
}
