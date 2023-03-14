use dashmap::DashMap;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use rspack_core::rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rspack_core::{runtime_globals, ChunkUkey, Compilation, RenderModuleContentArgs, SourceType};
use rspack_error::Result;

static MODULE_RENDER_CACHE: Lazy<DashMap<BoxSource, BoxSource>> = Lazy::new(DashMap::default);

pub fn render_chunk_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<BoxSource> {
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

  let plugin_driver = tokio::task::block_in_place(move || {
    tokio::runtime::Handle::current()
      .block_on(async move { compilation.plugin_driver.read().await })
  });

  let mut module_code_array = ordered_modules
    .par_iter()
    .filter(|mgm| mgm.used)
    .map(|mgm| {
      let result = compilation
        .code_generation_results
        .get(&mgm.module_identifier, Some(&chunk.runtime))
        .expect("should have code generation result")
        .get(&SourceType::JavaScript)
        .expect("should have js code generation result");

      let origin_source = result
        .ast_or_source
        .clone()
        .try_into_source()
        .expect("should be source");
      let module_source = if let Some(source) = plugin_driver
        .render_module_content(RenderModuleContentArgs {
          compilation,
          module_source: &origin_source,
        })
        .expect("render_module_content failed")
      {
        source
      } else {
        origin_source
      };

      // module id isn't cacheable
      let strict = match compilation
        .module_graph
        .module_by_identifier(&mgm.module_identifier)
        .and_then(|m| m.as_normal_module())
      {
        Some(normal_module) => normal_module.build_info.strict,
        None => false,
      };
      (
        mgm.module_identifier,
        render_module(module_source, strict, mgm.id(&compilation.chunk_graph)),
      )
    })
    .collect::<Vec<_>>();

  module_code_array.sort_unstable_by_key(|(module_identifier, _)| *module_identifier);

  let module_sources = module_code_array
    .into_par_iter()
    .fold(ConcatSource::default, |mut output, (_, source)| {
      output.add(source);
      output
    })
    .collect::<Vec<ConcatSource>>();

  let mut sources = ConcatSource::default();
  sources.add(RawSource::from("{\n"));
  sources.add(ConcatSource::new(module_sources));
  sources.add(RawSource::from("\n}"));

  Ok(sources.boxed())
}

pub fn render_module(source: BoxSource, strict: bool, module_id: &str) -> BoxSource {
  let mut sources = ConcatSource::new([
    RawSource::from("\""),
    RawSource::from(module_id.to_string()),
    RawSource::from("\": "),
    RawSource::from(format!(
      "function (module, exports, {}) {{\n",
      runtime_globals::REQUIRE
    )),
  ]);
  if strict {
    sources.add(RawSource::from("\"use strict\";\n"));
  }
  sources.add(source);
  sources.add(RawSource::from("},\n"));

  sources.boxed()
}

pub fn generate_chunk_entry_code(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> BoxSource {
  let entry_modules_uri = compilation.chunk_graph.get_chunk_entry_modules(chunk_ukey);
  let entry_modules_id = entry_modules_uri
    .into_iter()
    .filter_map(|entry_module_identifier| {
      compilation
        .module_graph
        .module_graph_module_by_identifier(&entry_module_identifier)
        .map(|module| module.id(&compilation.chunk_graph))
    })
    .collect::<Vec<_>>();
  let sources = entry_modules_id
    .iter()
    .map(|id| {
      RawSource::from(format!(
        "var __webpack_exports__ = {}('{}');\n",
        runtime_globals::REQUIRE,
        id
      ))
    })
    .collect::<Vec<_>>();
  ConcatSource::new(sources).boxed()
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
    runtime_globals::REQUIRE
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
    sources.add(RawSource::from(format!("// {}\n", module.identifier())));
    sources.add(RawSource::from("(function() {\n"));
    sources.add(source.clone());
    sources.add(RawSource::from("\n})();\n"));
  });
  Ok(sources.boxed())
}
