use crate::utils::{wrap_eval_source_map, wrap_module_function};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use rspack_core::rspack_sources::{BoxSource, CachedSource, ConcatSource, RawSource, SourceExt};
use rspack_core::{runtime_globals, ChunkUkey, Compilation, SourceType};
use rspack_error::Result;

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

  ordered_modules.sort_by_key(|m| &m.module_identifier);

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
          let mut module_source =
            CachedSource::new(result.ast_or_source.clone().try_into_source()?).boxed();
          if compilation.options.devtool.eval() && compilation.options.devtool.source_map() {
            let origin_source = module_source.clone();
            if let Some(cached) = MODULE_RENDER_CACHE.get(&origin_source) {
              module_source = cached.value().clone();
            } else {
              module_source = wrap_eval_source_map(module_source, compilation)?;
              MODULE_RENDER_CACHE.insert(origin_source, module_source.clone());
            }
          }
          // css or js same content isn't cacheable
          if mgm.module_type.is_css_like() && compilation.options.dev_server.hot {
            // inject css hmr runtime
            module_source = ConcatSource::new([
              module_source,
              RawSource::from(
                r#"
if (module.hot) {
  module.hot.accept();
}
"#,
              )
              .boxed(),
            ])
            .boxed();
          }
          // module id isn't cacheable
          Ok(wrap_module_function(
            module_source,
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
  sources.add(CachedSource::new(ConcatSource::new(module_sources)));
  sources.add(RawSource::from("\n}"));

  Ok(CachedSource::new(sources).boxed())
}

pub fn generate_chunk_entry_code(compilation: &Compilation, chunk_ukey: &ChunkUkey) -> BoxSource {
  let entry_modules_uri = compilation.chunk_graph.get_chunk_entry_modules(chunk_ukey);
  let entry_modules_id = entry_modules_uri
    .into_iter()
    .filter_map(|entry_module_identifier| {
      compilation
        .module_graph
        .module_graph_module_by_identifier(entry_module_identifier)
        .map(|module| module.id(&compilation.chunk_graph))
    })
    .collect::<Vec<_>>();
  // let namespace = &compilation.options.output.unique_name;
  let sources = entry_modules_id
    .iter()
    .map(|id| {
      if let Some(library) = &compilation.options.output.library && !library.is_empty() {
          RawSource::from(format!(r#"{} = {}("{}");"#, library, runtime_globals::REQUIRE, id))
        } else {
          RawSource::from(format!(r#"{}("{}");"#, runtime_globals::REQUIRE, id))
        }
    })
    .collect::<Vec<_>>();
  CachedSource::new(ConcatSource::new(sources)).boxed()
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
  Ok(CachedSource::new(sources).boxed())
}

pub fn render_runtime_modules(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<BoxSource> {
  let mut sources = ConcatSource::default();
  compilation
    .chunk_graph
    .get_chunk_runtime_modules_in_order(chunk_ukey)
    .iter()
    .filter_map(|identifier| compilation.runtime_modules.get(identifier))
    .for_each(|module| {
      sources.add(RawSource::from(format!("// {}\n", module.identifier())));
      sources.add(RawSource::from("(function() {\n"));
      sources.add(module.generate(compilation));
      sources.add(RawSource::from("\n})();\n"));
    });
  Ok(CachedSource::new(sources).boxed())
}
