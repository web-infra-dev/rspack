use dashmap::DashMap;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use rspack_core::rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, RawSource, SourceExt,
};
use rspack_core::{runtime_globals, ChunkUkey, Compilation, RuntimeModule, SourceType};
use rspack_error::Result;
use rspack_plugin_devtool::wrap_eval_source_map;

static MODULE_RENDER_CACHE: Lazy<DashMap<BoxSource, BoxSource>> = Lazy::new(DashMap::default);

pub fn render_chunk_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<BoxSource> {
  let module_graph = &compilation.module_graph;
  let mut ordered_modules = compilation.chunk_graph.get_chunk_modules_by_source_type(
    chunk_ukey,
    SourceType::JavaScript,
    module_graph,
  );
  let chunk = compilation
    .chunk_by_ukey
    .get(chunk_ukey)
    .expect("chunk not found");

  ordered_modules.sort_unstable_by_key(|m| &m.module_identifier);

  let module_code_array = ordered_modules
    .par_iter()
    .filter(|mgm| mgm.used)
    .map(|mgm| {
      let code_gen_result = compilation
        .code_generation_results
        .get(&mgm.module_identifier, Some(&chunk.runtime))?;

      code_gen_result
        .get(&SourceType::JavaScript)
        .map(|result| {
          let origin_source = result.ast_or_source.clone().try_into_source()?;
          let module_source =
            if compilation.options.devtool.eval() && compilation.options.devtool.source_map() {
              if let Some(cached) = MODULE_RENDER_CACHE.get(&origin_source) {
                cached.value().clone()
              } else {
                let module_source = if let Some(map) =
                  origin_source.map(&MapOptions::new(compilation.options.devtool.cheap()))
                {
                  wrap_eval_source_map(&origin_source.source(), map, compilation)?
                } else {
                  origin_source.clone()
                };
                let module_source = CachedSource::new(module_source).boxed();
                MODULE_RENDER_CACHE.insert(origin_source, module_source.clone());
                module_source
              }
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
          Ok(render_module(
            module_source,
            strict,
            mgm.id(&compilation.chunk_graph),
          ))
        })
        .transpose()
    })
    .collect::<Result<Vec<Option<BoxSource>>>>()?;

  let module_sources = module_code_array
    .into_par_iter()
    .flatten()
    .fold(ConcatSource::default, |mut output, cur| {
      output.add(cur);
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
  // let namespace = &compilation.options.output.unique_name;
  let sources = compilation
    .chunk_graph
    .get_chunk_entry_modules(chunk_ukey)
    .filter_map(|entry_module_identifier| {
      compilation
        .module_graph
        .module_graph_module_by_identifier(entry_module_identifier)
        .map(|module| {
          let id = module.id(&compilation.chunk_graph);
          if let Some(library) = &compilation.options.output.library && !library.is_empty() {
            RawSource::from(format!(r#"{} = {}("{}");"#, library, runtime_globals::REQUIRE, id))
          } else {
            RawSource::from(format!(r#"{}("{}");"#, runtime_globals::REQUIRE, id))
          }
        })
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
  let mut runtime_modules: Vec<&Box<dyn RuntimeModule>> = compilation
    .chunk_graph
    .get_chunk_runtime_modules_in_order(chunk_ukey)
    .iter()
    .filter_map(|identifier| compilation.runtime_modules.get(identifier))
    .collect();
  runtime_modules.sort_unstable_by_key(|a| a.stage());
  runtime_modules.iter().for_each(|module| {
    sources.add(RawSource::from(format!("// {}\n", module.identifier())));
    sources.add(RawSource::from("(function() {\n"));
    sources.add(module.generate(compilation));
    sources.add(RawSource::from("\n})();\n"));
  });
  Ok(sources.boxed())
}
