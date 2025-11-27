use rayon::prelude::*;
use rspack_core::{
  ChunkGraph, ChunkInitFragments, ChunkUkey, CodeGenerationPublicPathAutoReplace, Compilation,
  Module, RuntimeGlobals, SourceType,
  chunk_graph_chunk::ChunkId,
  get_undo_path,
  rspack_sources::{BoxSource, ConcatSource, RawStringSource, ReplaceSource, Source, SourceExt},
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rustc_hash::FxHashSet as HashSet;

use crate::{JavascriptModulesPluginHooks, RenderSource};

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_ASSET_AUTO_PUBLIC_PATH__";

pub async fn render_chunk_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  ordered_modules: &Vec<&dyn Module>,
  all_strict: bool,
  output_path: &str,
  hooks: &JavascriptModulesPluginHooks,
) -> Result<Option<(BoxSource, ChunkInitFragments)>> {
  let module_sources = rspack_futures::scope::<_, _>(|token| {
    ordered_modules.iter().for_each(|module| {
      let s = unsafe {
        token.used((
          compilation,
          chunk_ukey,
          module,
          all_strict,
          output_path,
          hooks,
        ))
      };
      s.spawn(
        |(compilation, chunk_ukey, module, all_strict, output_path, hooks)| async move {
          render_module(
            compilation,
            chunk_ukey,
            *module,
            all_strict,
            true,
            output_path,
            hooks,
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

pub async fn render_module(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &dyn Module,
  all_strict: bool,
  factory: bool,
  output_path: &str,
  hooks: &JavascriptModulesPluginHooks,
) -> Result<Option<(BoxSource, ChunkInitFragments, ChunkInitFragments)>> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
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
        replace.replace(start as u32, end as u32, &relative, None);
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

  hooks
    .render_module_content
    .call(
      compilation,
      chunk_ukey,
      module,
      &mut render_source,
      &mut module_chunk_init_fragments,
    )
    .await?;

  let sources = if factory {
    let mut sources = ConcatSource::default();
    let module_id =
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier())
        .expect("should have module_id in render_module");
    sources.add(RawStringSource::from(
      serde_json::to_string(&module_id).to_rspack_result()?,
    ));
    sources.add(RawStringSource::from_static(": "));

    let mut post_module_container = {
      let runtime_requirements = ChunkGraph::get_module_runtime_requirements(
        compilation,
        module.identifier(),
        chunk.runtime(),
      );

      let need_module = runtime_requirements.is_some_and(|r| r.contains(RuntimeGlobals::MODULE));
      let need_exports = runtime_requirements.is_some_and(|r| r.contains(RuntimeGlobals::EXPORTS));
      let need_require = runtime_requirements.is_some_and(|r| {
        r.contains(RuntimeGlobals::REQUIRE) || r.contains(RuntimeGlobals::REQUIRE_SCOPE)
      });

      let mut args = Vec::new();
      if need_module || need_exports || need_require {
        let module_argument = compilation
          .runtime_template
          .render_module_argument(module.get_module_argument());
        args.push(if need_module {
          module_argument
        } else {
          format!("__unused_webpack_{module_argument}")
        });
      }

      if need_exports || need_require {
        let exports_argument = compilation
          .runtime_template
          .render_exports_argument(module.get_exports_argument());
        args.push(if need_exports {
          exports_argument
        } else {
          format!("__unused_webpack_{exports_argument}")
        });
      }
      if need_require {
        args.push(
          compilation
            .runtime_template
            .render_runtime_globals(&RuntimeGlobals::REQUIRE),
        );
      }

      let mut container_sources = ConcatSource::default();

      container_sources.add(RawStringSource::from(format!(
        "(function ({}) {{\n",
        args.join(", ")
      )));
      if module.build_info().strict && !all_strict {
        container_sources.add(RawStringSource::from_static("\"use strict\";\n"));
      }
      container_sources.add(render_source.source);
      container_sources.add(RawStringSource::from_static("\n\n})"));
      container_sources.add(RawStringSource::from_static(",\n"));

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
) -> Result<BoxSource> {
  let runtime_modules_sources = render_runtime_modules(compilation, chunk_ukey).await?;
  if runtime_modules_sources.source().is_empty() {
    return Ok(runtime_modules_sources);
  }

  let mut sources = ConcatSource::default();
  sources.add(RawStringSource::from(format!(
    "function({}) {{\n",
    compilation
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::REQUIRE),
  )));
  sources.add(runtime_modules_sources);
  sources.add(RawStringSource::from_static("\n}\n"));
  Ok(sources.boxed())
}

pub async fn render_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<BoxSource> {
  let mut sources = ConcatSource::default();
  let runtime_module_sources = rspack_futures::scope::<_, Result<_>>(|token| {
    compilation
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
        s.spawn(|(compilation, source, module)| async move {
          let mut sources = ConcatSource::default();
          if source.size() == 0 {
            return Ok(sources);
          }
          sources.add(RawStringSource::from(format!(
            "// {}\n",
            module.identifier()
          )));
          let supports_arrow_function = compilation
            .options
            .output
            .environment
            .supports_arrow_function();
          if module.should_isolate() {
            sources.add(RawStringSource::from(if supports_arrow_function {
              "(() => {\n"
            } else {
              "!function() {\n"
            }));
          }
          if !(module.full_hash() || module.dependent_hash()) {
            sources.add(source.clone());
          } else {
            let result = module.code_generation(compilation, None, None).await?;
            #[allow(clippy::unwrap_used)]
            let source = result.get(&SourceType::Runtime).unwrap();
            sources.add(source.clone());
          }
          if module.should_isolate() {
            sources.add(RawStringSource::from(if supports_arrow_function {
              "\n})();\n"
            } else {
              "\n}();\n"
            }));
          }
          Ok(sources)
        });
      })
  })
  .await
  .into_iter()
  .map(|r| r.to_rspack_result())
  .collect::<Result<Vec<_>>>()?;

  for runtime_module_source in runtime_module_sources {
    sources.add(runtime_module_source?);
  }

  Ok(sources.boxed())
}

pub fn stringify_chunks_to_array(chunks: &HashSet<ChunkId>) -> String {
  let mut v = Vec::from_iter(chunks.iter());
  v.sort_unstable();

  format!(
    r#"[{}]"#,
    v.iter().fold(String::new(), |prev, cur| {
      prev + format!(r#""{cur}","#).as_str()
    })
  )
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
