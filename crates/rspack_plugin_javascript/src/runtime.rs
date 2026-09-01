use cow_utils::CowUtils;
use rayon::prelude::*;
use rspack_core::{
  ChunkGraph, ChunkInitFragments, ChunkKind, ChunkUkey, CodeGenerationDataChunkInitFragments,
  CodeGenerationDataPreservedAssetImport, CodeGenerationPublicPathAutoReplace, Compilation,
  ExternalModuleInitFragment, InitFragmentExt, InitFragmentStage, Module, RuntimeCodeTemplate,
  RuntimeGlobals, RuntimeGlobalsRenderMode, RuntimeModuleGenerateContext, SourceType,
  chunk_graph_chunk::ChunkIdSet,
  get_undo_path, render_runtime_module_source,
  rspack_sources::{
    BoxSource, ConcatSource, OriginalSource, RawStringSource, ReplaceSource, Source, SourceExt,
  },
  runtime_mode::RuntimeMode,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};

pub use crate::runtime_context::{
  render_hot_update_chunk_runtime_modules as render_rspack_hot_update_chunk_runtime_modules,
  render_rspack_runtime_modules,
  render_runtime_chunk_runtime_modules as render_rspack_runtime_chunk_runtime_modules,
  render_runtime_context_declaration, render_runtime_context_require_assignment,
  should_export_rspack_runtime_globals,
};
use crate::{JavascriptModulesPluginHooks, RenderSource};

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_ASSET_AUTO_PUBLIC_PATH__";

pub async fn render_chunk_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  ordered_modules: &Vec<&dyn Module>,
  all_strict: bool,
  output_path: &str,
  hooks: &JavascriptModulesPluginHooks,
  runtime_template: &RuntimeCodeTemplate,
) -> Result<Option<(BoxSource, ChunkInitFragments)>> {
  let module_runtime_scope = compilation
    .runtime_template
    .create_module_code_template()
    .render_runtime_scope();
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
          runtime_template,
          module_runtime_scope.as_str(),
        ))
      };
      s.spawn(
        |(
          compilation,
          chunk_ukey,
          module,
          all_strict,
          output_path,
          hooks,
          runtime_template,
          module_runtime_scope,
        )| async move {
          render_module(
            compilation,
            chunk_ukey,
            *module,
            all_strict,
            true,
            true,
            output_path,
            hooks,
            runtime_template,
            Some(module_runtime_scope),
          )
          .await
          .map(|result| result.map(|(source, fragments)| (module.identifier(), source, fragments)))
        },
      );
    });
  })
  .await
  .into_iter()
  .map(|r| r.to_rspack_result())
  .collect::<Result<Vec<_>>>()?;

  let mut module_code_array = Vec::with_capacity(module_sources.len());
  for item in module_sources {
    if let Some(i) = item? {
      module_code_array.push(i);
    }
  }

  if module_code_array.is_empty() {
    return Ok(None);
  }

  module_code_array.sort_unstable_by_key(|(module_identifier, _, _)| *module_identifier);

  let mut chunk_init_fragments = ChunkInitFragments::default();
  let mut module_sources = Vec::with_capacity(module_code_array.len());
  for (_, source, fragments) in module_code_array {
    chunk_init_fragments.extend(fragments);
    module_sources.push(source);
  }
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
  render_preserved_asset_import_fragment: bool,
  output_path: &str,
  hooks: &JavascriptModulesPluginHooks,
  runtime_template: &RuntimeCodeTemplate,
  module_runtime_scope: Option<&str>,
) -> Result<Option<(BoxSource, ChunkInitFragments)>> {
  let module_identifier = module.identifier();
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  let code_gen_result = compilation
    .code_generation_results
    .get(&module_identifier, Some(chunk.runtime()));
  let Some(origin_source) = code_gen_result.get(&SourceType::JavaScript) else {
    return Ok(None);
  };

  let mut module_chunk_init_fragments = code_gen_result
    .data()
    .get::<CodeGenerationDataChunkInitFragments>()
    .map(|fragments| fragments.inner().clone())
    .unwrap_or_default();
  if render_preserved_asset_import_fragment
    && let Some(asset_import) = code_gen_result
      .data()
      .get::<CodeGenerationDataPreservedAssetImport>()
  {
    // The normal JavaScript renderer has no chunk linker for raw imports. Use the same structured
    // fragment as ExternalModuleDependency, after the final output-relative request is known.
    let relative = get_undo_path(
      output_path,
      compilation.options.output.path.to_string(),
      true,
    );
    let request = asset_import
      .request()
      .cow_replace(AUTO_PUBLIC_PATH_PLACEHOLDER, &relative)
      .into_owned();
    let position = compilation
      .get_module_graph()
      .get_pre_order_index(&module_identifier)
      .map_or(0, |index| index as i32);
    module_chunk_init_fragments.push(
      ExternalModuleInitFragment::new(
        request,
        Vec::new(),
        Some(asset_import.binding().to_string()),
        InitFragmentStage::StageESMImports,
        position,
      )
      .boxed(),
    );
  }
  let mut render_runtime_requirements = *code_gen_result.runtime_requirements();

  let mut render_source = if code_gen_result
    .data()
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
      &mut render_runtime_requirements,
      &mut module_chunk_init_fragments,
      runtime_template,
    )
    .await?;

  let sources = if factory {
    let mut sources = ConcatSource::default();
    let module_id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, module_identifier)
      .expect("should have module_id in render_module");
    sources.add(RawStringSource::from(rspack_util::json_stringify(
      module_id,
    )));

    let mut post_module_container = {
      let runtime_requirements = ChunkGraph::get_module_runtime_requirements(
        compilation,
        module_identifier,
        chunk.runtime(),
      );

      let need_module = runtime_requirements.is_some_and(|r| r.contains(RuntimeGlobals::MODULE));
      let need_exports = runtime_requirements.is_some_and(|r| r.contains(RuntimeGlobals::EXPORTS));
      let need_require = runtime_template.render_mode() != RuntimeGlobalsRenderMode::RspackExport
        && (render_runtime_requirements.contains(RuntimeGlobals::REQUIRE)
          || render_runtime_requirements.contains(RuntimeGlobals::REQUIRE_SCOPE)
          || !render_runtime_requirements
            .renderable_require_scope()
            .is_empty());
      let mut args = String::with_capacity(64);
      let mut push_argument = |argument: &str, used: bool| {
        if !args.is_empty() {
          args.push_str(", ");
        }
        if !used {
          args.push_str("__unused_rspack_");
        }
        args.push_str(argument);
      };
      if need_module || need_exports || need_require {
        let module_argument = runtime_template.render_module_argument(module.get_module_argument());
        push_argument(&module_argument, need_module);
      }

      if need_exports || need_require {
        let exports_argument =
          runtime_template.render_exports_argument(module.get_exports_argument());
        push_argument(&exports_argument, need_exports);
      }
      if need_require {
        push_argument(
          module_runtime_scope
            .expect("module runtime scope should be provided for factory modules"),
          true,
        );
      }

      let mut container_sources = ConcatSource::default();

      let mut container_prefix = String::with_capacity(args.len() + 16);
      if use_method_shorthand {
        container_prefix.push('(');
        container_prefix.push_str(&args);
        container_prefix.push_str(") {\n");
      } else {
        container_prefix.push_str(": (function (");
        container_prefix.push_str(&args);
        container_prefix.push_str(") {\n");
      }
      container_sources.add(RawStringSource::from(container_prefix));
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

  Ok(Some((sources, module_chunk_init_fragments)))
}

pub async fn render_chunk_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &RuntimeCodeTemplate,
) -> Result<BoxSource> {
  let runtime_modules_sources =
    if compilation.options.experiments.runtime_mode == RuntimeMode::Rspack {
      let chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(chunk_ukey);
      if matches!(chunk.kind(), ChunkKind::HotUpdate) {
        crate::runtime_context::render_hot_update_chunk_runtime_modules(
          compilation,
          chunk_ukey,
          runtime_template,
        )
        .await
      } else if chunk.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey) {
        crate::runtime_context::render_runtime_chunk_runtime_modules(
          compilation,
          chunk_ukey,
          runtime_template,
        )
        .await
      } else {
        crate::runtime_context::render_chunk_runtime_modules(
          compilation,
          chunk_ukey,
          runtime_template,
        )
        .await
      }
    } else {
      render_runtime_modules(compilation, chunk_ukey, runtime_template).await
    }?;
  if runtime_modules_sources.source().is_empty() {
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
  runtime_template: &RuntimeCodeTemplate,
) -> Result<BoxSource> {
  if compilation.options.experiments.runtime_mode == RuntimeMode::Rspack {
    render_rspack_runtime_modules(compilation, chunk_ukey, runtime_template).await
  } else {
    render_webpack_runtime_modules(compilation, chunk_ukey).await
  }
}

pub(crate) type RuntimeModuleSourceItem = (BoxSource, RuntimeGlobals, RuntimeGlobals, bool);

pub(crate) async fn render_runtime_module_sources(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  reject_custom_runtime_modules: bool,
) -> Result<Vec<RuntimeModuleSourceItem>> {
  let runtime_mode = compilation.options.experiments.runtime_mode;
  let runtime_module_sources = rspack_parallel::scope::<_, Result<_>>(|token| {
    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_runtime_modules_in_order(chunk_ukey, compilation)
      .map(|(identifier, runtime_module)| {
        (
          compilation
            .runtime_modules_code_generation_source
            .get(identifier)
            .expect("should have runtime module result"),
          runtime_module,
        )
      })
      .for_each(|(source, module)| {
        let s = unsafe { token.used((compilation, source, module)) };
        s.spawn(
          move |(compilation, source, module)| async move {
            if source.size() == 0 {
              return Ok((
                ConcatSource::default().boxed(),
                RuntimeGlobals::default(),
                RuntimeGlobals::default(),
                false,
              ));
            }
            let runtime_requirements = module.runtime_requirements(compilation);
            let generated_requirements = runtime_requirements.lexical_requirements();
            let context_requirements =
              runtime_requirements.define | runtime_requirements.force_context;
            if reject_custom_runtime_modules
              && module.get_constructor_name() == "RuntimeModuleFromJs"
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
            let source = if !(module.full_hash() || module.dependent_hash()) {
              if let Some(custom_source) = module.get_custom_source() {
                RawStringSource::from(custom_source).boxed()
              } else {
                source.clone()
              }
            } else {
              if let Some(custom_source) = module.get_custom_source() {
                RawStringSource::from(custom_source).boxed()
              } else {
                let runtime_template = compilation.runtime_template.create_runtime_module_code_template();
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
            let should_isolate = module.should_isolate(runtime_mode);
            let needs_top_level = matches!(runtime_mode, RuntimeMode::Rspack)
              && module.get_constructor_name() == "ExportRequireRuntimeModule";
            let sources = render_runtime_module_source(
              module.identifier(),
              source,
              should_isolate,
              supports_arrow_function,
              matches!(runtime_mode, RuntimeMode::Rspack) && !should_isolate,
            );
            Ok((
              sources,
              generated_requirements,
              context_requirements,
              needs_top_level,
            ))
          },
        );
      })
  })
  .await
  .into_iter()
  .map(|r| r.to_rspack_result().and_then(|result| result))
  .collect::<Result<Vec<_>>>()?;

  Ok(runtime_module_sources)
}

async fn render_webpack_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<BoxSource> {
  let runtime_module_sources =
    render_runtime_module_sources(compilation, chunk_ukey, false).await?;
  let mut sources = ConcatSource::default();

  for (runtime_module_source, _, _, _) in runtime_module_sources {
    sources.add(runtime_module_source);
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
