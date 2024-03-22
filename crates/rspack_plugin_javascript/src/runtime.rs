use rayon::prelude::*;
use rspack_core::rspack_sources::{BoxSource, ConcatSource, RawSource, Source, SourceExt};
use rspack_core::{
  BoxModule, Chunk, ChunkGraph, ChunkInitFragments, ChunkUkey, Compilation, Context, ModuleGraph,
  PluginDriver, RenderModuleContentArgs, RenderModulePackageContext, RuntimeGlobals, SourceType,
};
use rspack_error::{error, Result};
use rustc_hash::FxHashSet as HashSet;

use crate::utils::is_diff_mode;

pub fn render_chunk_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<(BoxSource, ChunkInitFragments)> {
  let module_graph = &compilation.get_module_graph();
  let ordered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
    chunk_ukey,
    SourceType::JavaScript,
    module_graph,
  );
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  let plugin_driver = &compilation.plugin_driver;

  let include_module_ids = &compilation.include_module_ids;

  let mut module_code_array = ordered_modules
    .par_iter()
    .filter(|module| {
      compilation.get_module_graph().is_new_treeshaking()
        || include_module_ids.contains(&module.identifier())
    })
    .filter_map(|module| {
      let code_gen_result = compilation
        .code_generation_results
        .get(&module.identifier(), Some(&chunk.runtime));
      if let Some(origin_source) = code_gen_result.get(&SourceType::JavaScript) {
        let render_module_result = plugin_driver
          .render_module_content(RenderModuleContentArgs {
            compilation,
            module,
            module_source: origin_source.clone(),
            chunk_init_fragments: ChunkInitFragments::default(),
          })
          .expect("render_module_content failed");

        let runtime_requirements = compilation
          .chunk_graph
          .get_module_runtime_requirements(module.identifier(), &chunk.runtime);

        Some((
          module.identifier(),
          render_module(
            render_module_result.module_source,
            chunk,
            module,
            runtime_requirements,
            compilation
              .chunk_graph
              .get_module_id(module.identifier())
              .as_ref()
              .expect("should have module id"),
            &compilation.options.context,
            compilation.get_module_graph(),
            &compilation.chunk_graph,
            &compilation.plugin_driver,
          ),
          &code_gen_result.chunk_init_fragments,
          render_module_result.chunk_init_fragments,
        ))
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

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
    .collect::<Result<_>>()?;
  let module_sources = module_sources
    .into_par_iter()
    .fold(ConcatSource::default, |mut output, source| {
      output.add(source);
      output
    })
    .collect::<Vec<ConcatSource>>();

  let mut sources = ConcatSource::default();
  sources.add(RawSource::from("{\n"));
  sources.add(ConcatSource::new(module_sources));
  sources.add(RawSource::from("\n}"));

  Ok((sources.boxed(), chunk_init_fragments))
}

fn render_module(
  source: BoxSource,
  chunk: &Chunk,
  module: &BoxModule,
  runtime_requirements: Option<&RuntimeGlobals>,
  module_id: &str,
  context: &Context,
  module_graph: &ModuleGraph,
  chunk_graph: &ChunkGraph,
  plugin_driver: &PluginDriver,
) -> Result<BoxSource> {
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
  let mut sources = ConcatSource::new([
    RawSource::from(serde_json::to_string(module_id).map_err(|e| error!(e.to_string()))?),
    RawSource::from(": "),
  ]);
  if is_diff_mode() {
    sources.add(RawSource::from(format!("\n/* start::{} */\n", module_id)));
  }
  sources.add(RawSource::from(format!(
    "(function ({}) {{\n",
    args.join(", ")
  )));
  if let Some(build_info) = &module.build_info()
    && build_info.strict
  {
    sources.add(RawSource::from("\"use strict\";\n"));
  }
  sources.add(source);
  sources.add(RawSource::from("})"));
  if is_diff_mode() {
    sources.add(RawSource::from(format!("\n/* end::{} */\n", module_id)));
  }
  sources.add(RawSource::from(",\n"));

  let render_module_package_args = RenderModulePackageContext {
    chunk,
    context,
    module_graph,
    chunk_graph,
  };
  plugin_driver
    .render_module_package(
      sources.boxed(),
      module.as_ref(),
      &render_module_package_args,
    )
    .map(|source| source.boxed())
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
  sources.add(RawSource::from(format!(
    "function({}) {{\n",
    RuntimeGlobals::REQUIRE
  )));
  sources.add(runtime_modules_sources);
  sources.add(RawSource::from("\n}\n"));
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
          .runtime_module_code_generation_results
          .get(identifier)
          .expect("should have runtime module result"),
        runtime_module,
      )
    })
    .for_each(|((_, source), module)| {
      if source.size() == 0 {
        return;
      }
      if is_diff_mode() {
        sources.add(RawSource::from(format!(
          "/* start::{} */\n",
          module.identifier()
        )));
      } else {
        sources.add(RawSource::from(format!("// {}\n", module.identifier())));
      }
      if !module.should_isolate() {
        sources.add(RawSource::from("!function() {\n"));
      }
      if module.cacheable() {
        sources.add(source.clone());
      } else {
        sources.add(module.generate_with_custom(compilation));
      }
      if !module.should_isolate() {
        sources.add(RawSource::from("\n}();\n"));
      }
      if is_diff_mode() {
        sources.add(RawSource::from(format!(
          "/* end::{} */\n",
          module.identifier()
        )));
      }
    });
  Ok(sources.boxed())
}

pub fn stringify_chunks_to_array(chunks: &HashSet<String>) -> String {
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

pub fn render_iife(content: BoxSource) -> BoxSource {
  let mut sources = ConcatSource::default();
  sources.add(RawSource::from("(function() {\n"));
  sources.add(content);
  sources.add(RawSource::from("\n})()\n"));
  sources.boxed()
}
