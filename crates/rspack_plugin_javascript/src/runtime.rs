use rayon::prelude::*;
use rspack_core::rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rspack_core::{
  ChunkInitFragments, ChunkUkey, Compilation, ModuleGraphModule, RenderModuleContentArgs,
  RuntimeGlobals, SourceType,
};
use rspack_error::{internal_error, Result};
use rustc_hash::FxHashSet as HashSet;

use crate::utils::is_diff_mode;

pub fn render_chunk_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<(BoxSource, ChunkInitFragments)> {
  let module_graph = &compilation.module_graph;
  let ordered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
    chunk_ukey,
    SourceType::JavaScript,
    module_graph,
  );
  let chunk = compilation
    .chunk_by_ukey
    .get(chunk_ukey)
    .expect("chunk not found");

  let plugin_driver = &compilation.plugin_driver;

  let include_module_ids = &compilation.include_module_ids;

  let mut module_code_array = ordered_modules
    .par_iter()
    .filter(|mgm| include_module_ids.contains(&mgm.module_identifier))
    .filter_map(|mgm| {
      let code_gen_result = compilation
        .code_generation_results
        .get(&mgm.module_identifier, Some(&chunk.runtime))
        .expect("should have code generation result");
      if let Some(origin_source) = code_gen_result.get(&SourceType::JavaScript) {
        let render_module_result = plugin_driver
          .render_module_content(RenderModuleContentArgs {
            compilation,
            module_graph_module: mgm,
            module_source: origin_source.clone(),
            chunk_init_fragments: ChunkInitFragments::default(),
          })
          .expect("render_module_content failed");

        let runtime_requirements = compilation
          .chunk_graph
          .get_module_runtime_requirements(mgm.module_identifier, &chunk.runtime);

        Some((
          mgm.module_identifier,
          render_module(
            render_module_result.module_source,
            mgm,
            runtime_requirements,
            mgm.id(&compilation.chunk_graph),
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
  mgm: &ModuleGraphModule,
  runtime_requirements: Option<&RuntimeGlobals>,
  module_id: &str,
) -> Result<BoxSource> {
  // TODO unused exports_argument
  let module_argument = {
    let module_argument = mgm.get_module_argument();
    if let Some(runtime_requirements) = runtime_requirements
      && runtime_requirements.contains(RuntimeGlobals::MODULE)
    {
      module_argument.to_string()
    } else {
      format!("__unused_webpack_{module_argument}")
    }
  };
  let exports_argument = mgm.get_exports_argument();
  let mut sources = ConcatSource::new([
    RawSource::from(serde_json::to_string(module_id).map_err(|e| internal_error!(e.to_string()))?),
    RawSource::from(": "),
  ]);
  if is_diff_mode() {
    sources.add(RawSource::from(format!("\n/* start::{} */\n", module_id)));
  }
  sources.add(RawSource::from(format!(
    "(function ({module_argument}, {exports_argument}, {}) {{\n",
    RuntimeGlobals::REQUIRE
  )));
  if let Some(build_info) = &mgm.build_info
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

  Ok(sources.boxed())
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
  let mut runtime_modules = compilation
    .chunk_graph
    .get_chunk_runtime_modules_in_order(chunk_ukey)
    .iter()
    .map(|identifier| {
      (
        compilation
          .runtime_module_code_generation_results
          .get(identifier)
          .expect("should have runtime module result"),
        compilation
          .runtime_modules
          .get(identifier)
          .expect("should have runtime module"),
      )
    })
    .collect::<Vec<_>>();
  runtime_modules.sort_unstable_by_key(|(_, m)| m.stage());
  runtime_modules.iter().for_each(|((_, source), module)| {
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
      sources.add(module.generate(compilation));
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
    vec.iter().fold(String::new(), |prev, cur| {
      prev + format!(r#""{cur}","#).as_str()
    })
  )
}

pub fn render_iife(content: BoxSource) -> BoxSource {
  let mut sources = ConcatSource::default();
  sources.add(RawSource::from("(function() {\n"));
  sources.add(content);
  sources.add(RawSource::from("\n})()\n"));
  sources.boxed()
}
