use rayon::prelude::*;
use rspack_core::chunk_graph_chunk::ChunkId;
use rspack_core::rspack_sources::{
  BoxSource, ConcatSource, RawStringSource, ReplaceSource, Source, SourceExt,
};
use rspack_core::{
  get_undo_path, to_normal_comment, BoxModule, ChunkGraph, ChunkInitFragments, ChunkUkey,
  CodeGenerationPublicPathAutoReplace, Compilation, RuntimeGlobals, SourceType,
};
use rspack_error::{error, Result};
use rspack_util::diff_mode::is_diff_mode;
use rustc_hash::FxHashSet as HashSet;

use crate::{JsPlugin, RenderSource};

pub const AUTO_PUBLIC_PATH_PLACEHOLDER: &str = "__RSPACK_PLUGIN_ASSET_AUTO_PUBLIC_PATH__";

pub fn render_chunk_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  ordered_modules: &Vec<&BoxModule>,
  all_strict: bool,
  output_path: &str,
) -> Result<Option<(BoxSource, ChunkInitFragments)>> {
  let mut module_code_array = ordered_modules
    .par_iter()
    .filter_map(|module| {
      render_module(
        compilation,
        chunk_ukey,
        module,
        all_strict,
        true,
        output_path,
      )
      .transpose()
      .map(|result| result.map(|(s, f, a)| (module.identifier(), s, f, a)))
    })
    .collect::<Result<Vec<_>>>()?;

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

pub fn render_module(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &BoxModule,
  all_strict: bool,
  factory: bool,
  output_path: &str,
) -> Result<Option<(BoxSource, ChunkInitFragments, ChunkInitFragments)>> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let code_gen_result = compilation
    .code_generation_results
    .get(&module.identifier(), Some(chunk.runtime()));
  let Some(origin_source) = code_gen_result.get(&SourceType::JavaScript) else {
    return Ok(None);
  };

  let hooks = JsPlugin::get_compilation_hooks(compilation.id());
  let mut module_chunk_init_fragments = match code_gen_result.data.get::<ChunkInitFragments>() {
    Some(fragments) => fragments.clone(),
    None => ChunkInitFragments::default(),
  };

  let mut render_source = if code_gen_result
    .data
    .get::<CodeGenerationPublicPathAutoReplace>()
    .is_some()
  {
    let content = origin_source.source();
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

  hooks.render_module_content.call(
    compilation,
    module,
    &mut render_source,
    &mut module_chunk_init_fragments,
  )?;

  let sources = if factory {
    let mut sources = ConcatSource::default();
    let module_id =
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier())
        .expect("should have module_id in render_module");
    sources.add(RawStringSource::from(
      serde_json::to_string(&module_id).map_err(|e| error!(e.to_string()))?,
    ));
    sources.add(RawStringSource::from_static(": "));

    if is_diff_mode() {
      sources.add(RawStringSource::from(format!(
        "\n{}\n",
        to_normal_comment(&format!("start::{}", module.identifier()))
      )));
    }

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
        let module_argument = module.get_module_argument();
        args.push(if need_module {
          module_argument.to_string()
        } else {
          format!("__unused_webpack_{module_argument}")
        });
      }

      if need_exports || need_require {
        let exports_argument = module.get_exports_argument();
        args.push(if need_exports {
          exports_argument.to_string()
        } else {
          format!("__unused_webpack_{exports_argument}")
        });
      }
      if need_require {
        args.push(RuntimeGlobals::REQUIRE.to_string());
      }

      let mut container_sources = ConcatSource::default();

      // TODO: put this in a plugin via render_module_{container,package} hook
      if is_diff_mode() {
        container_sources.add(RawStringSource::from(format!(
          "\n{}\n",
          to_normal_comment(&format!("start::{}", module.identifier()))
        )));
      }

      container_sources.add(RawStringSource::from(format!(
        "(function ({}) {{\n",
        args.join(", ")
      )));
      if module.build_info().strict && !all_strict {
        container_sources.add(RawStringSource::from_static("\"use strict\";\n"));
      }
      container_sources.add(render_source.source);
      container_sources.add(RawStringSource::from_static("\n\n})"));

      if is_diff_mode() {
        container_sources.add(RawStringSource::from(format!(
          "\n{}\n",
          to_normal_comment(&format!("end::{}", module.identifier()))
        )));
      }
      container_sources.add(RawStringSource::from_static(",\n"));

      RenderSource {
        source: container_sources.boxed(),
      }
    };

    hooks.render_module_container.call(
      compilation,
      module,
      &mut post_module_container,
      &mut module_chunk_init_fragments,
    )?;

    let mut post_module_package = post_module_container;

    hooks.render_module_package.call(
      compilation,
      chunk_ukey,
      module,
      &mut post_module_package,
      &mut module_chunk_init_fragments,
    )?;

    sources.add(post_module_package.source);
    sources.boxed()
  } else {
    hooks.render_module_package.call(
      compilation,
      chunk_ukey,
      module,
      &mut render_source,
      &mut module_chunk_init_fragments,
    )?;

    render_source.source
  };

  Ok(Some((
    sources,
    code_gen_result.chunk_init_fragments.clone(),
    module_chunk_init_fragments,
  )))
}

pub fn render_chunk_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<BoxSource> {
  let runtime_modules_sources = render_runtime_modules(compilation, chunk_ukey)?;
  if runtime_modules_sources.source().is_empty() {
    return Ok(runtime_modules_sources);
  }

  let mut sources = ConcatSource::default();
  sources.add(RawStringSource::from(format!(
    "function({}) {{\n",
    RuntimeGlobals::REQUIRE
  )));
  sources.add(runtime_modules_sources);
  sources.add(RawStringSource::from_static("\n}\n"));
  Ok(sources.boxed())
}

pub fn render_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<BoxSource> {
  let mut sources = ConcatSource::default();
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
    .try_for_each(|(source, module)| -> Result<()> {
      if source.size() == 0 {
        return Ok(());
      }
      if is_diff_mode() {
        sources.add(RawStringSource::from(format!(
          "/* start::{} */\n",
          module.identifier()
        )));
      } else {
        sources.add(RawStringSource::from(format!(
          "// {}\n",
          module.identifier()
        )));
      }
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
        sources.add(module.generate_with_custom(compilation)?);
      }
      if module.should_isolate() {
        sources.add(RawStringSource::from(if supports_arrow_function {
          "\n})();\n"
        } else {
          "\n}();\n"
        }));
      }
      if is_diff_mode() {
        sources.add(RawStringSource::from(format!(
          "/* end::{} */\n",
          module.identifier()
        )));
      }
      Ok(())
    })?;
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
