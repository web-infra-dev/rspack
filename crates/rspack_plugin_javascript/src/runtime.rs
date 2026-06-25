use rayon::prelude::*;
use rspack_core::{
  ChunkCodeTemplate, ChunkGraph, ChunkInitFragments, ChunkKind, ChunkUkey,
  CodeGenerationPublicPathAutoReplace, Compilation, Module, RuntimeGlobals,
  RuntimeModuleGenerateContext, SourceType,
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
      let need_require = if need_require {
        render_source
          .source
          .source()
          .into_string_lossy()
          .contains(&runtime_template.render_runtime_argument())
      } else {
        need_require
      };

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

pub async fn render_chunk_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
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
  runtime_template: &ChunkCodeTemplate,
) -> Result<BoxSource> {
  if compilation.options.experiments.runtime_mode == RuntimeMode::Rspack {
    render_rspack_runtime_modules(compilation, chunk_ukey, runtime_template).await
  } else {
    render_webpack_runtime_modules(compilation, chunk_ukey, runtime_template).await
  }
}

pub(crate) type RuntimeModuleSourceItem = (BoxSource, RuntimeGlobals, RuntimeGlobals);

pub(crate) async fn render_runtime_module_sources(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_template: &ChunkCodeTemplate,
  reject_custom_runtime_modules: bool,
  isolate_runtime_modules: bool,
) -> Result<Vec<RuntimeModuleSourceItem>> {
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
        let s = unsafe { token.used((compilation, source, module, runtime_template)) };
        s.spawn(
          move |(compilation, source, module, runtime_template)| async move {
            if source.size() == 0 {
              return Ok((
                ConcatSource::default().boxed(),
                RuntimeGlobals::default(),
                RuntimeGlobals::default(),
              ));
            }
            let runtime_requirements = module.runtime_requirements(compilation);
            let generated_requirements = runtime_requirements.lexical_requirements();
            let context_requirements =
              runtime_requirements.write | runtime_requirements.force_context;
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
              isolate_runtime_modules && module.should_isolate(),
              supports_arrow_function,
            );
            Ok((sources, generated_requirements, context_requirements))
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
  runtime_template: &ChunkCodeTemplate,
) -> Result<BoxSource> {
  let runtime_module_sources =
    render_runtime_module_sources(compilation, chunk_ukey, runtime_template, false, true).await?;
  let mut sources = ConcatSource::default();

  for (runtime_module_source, _, _) in runtime_module_sources {
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
