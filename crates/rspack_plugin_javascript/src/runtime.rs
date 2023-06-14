use rayon::prelude::*;
use rspack_core::rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt};
use rspack_core::{
  ChunkInitFragments, ChunkUkey, Compilation, InitFragment, RenderModuleContentArgs,
  RuntimeGlobals, SourceType,
};
use rspack_error::{internal_error, Result};
use rustc_hash::FxHashSet as HashSet;

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
      if let Some(result) = code_gen_result.get(&SourceType::JavaScript) {
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
        let strict = mgm
          .build_info
          .as_ref()
          .map(|m| m.strict)
          .unwrap_or_default();
        Some((
          mgm.module_identifier,
          render_module(module_source, strict, mgm.id(&compilation.chunk_graph)),
          &code_gen_result.chunk_init_fragments,
        ))
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

  module_code_array.sort_unstable_by_key(|(module_identifier, _, _)| *module_identifier);

  let chunk_init_fragments = module_code_array.iter().fold(
    ChunkInitFragments::default(),
    |mut chunk_init_fragments, (_, _, fragments)| {
      for (k, v) in fragments.iter() {
        chunk_init_fragments.insert(k.to_string(), v.clone());
      }
      chunk_init_fragments
    },
  );

  let module_sources: Vec<_> = module_code_array
    .into_iter()
    .map(|(_, source, _)| source)
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

/* remove `strict` parameter for now, let SWC manage `use strict` annotation directly */
fn render_module(source: BoxSource, _strict: bool, module_id: &str) -> Result<BoxSource> {
  let mut sources = ConcatSource::new([
    RawSource::from(serde_json::to_string(module_id).map_err(|e| internal_error!(e.to_string()))?),
    RawSource::from(": "),
    RawSource::from(format!(
      "function (module, exports, {}) {{\n",
      RuntimeGlobals::REQUIRE
    )),
  ]);
  // if strict {
  //   sources.add(RawSource::from("\"use strict\";\n"));
  // }
  sources.add(source);
  sources.add(RawSource::from("},\n"));

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
    sources.add(RawSource::from(format!("// {}\n", module.identifier())));
    if !module.should_isolate() {
      sources.add(RawSource::from("(function() {\n"));
    }
    if module.cacheable() {
      sources.add(source.clone());
    } else {
      sources.add(module.generate(compilation));
    }
    if !module.should_isolate() {
      sources.add(RawSource::from("\n})();\n"));
    }
  });
  Ok(sources.boxed())
}

pub fn render_chunk_init_fragments(
  source: BoxSource,
  chunk_init_fragments: ChunkInitFragments,
) -> BoxSource {
  let mut fragments = chunk_init_fragments.into_values().collect::<Vec<_>>();
  render_init_fragments(source, &mut fragments)
}

pub fn render_init_fragments(source: BoxSource, fragments: &mut [InitFragment]) -> BoxSource {
  // here use sort_by_key because need keep order equal stage fragments
  fragments.sort_by_key(|m| m.stage);

  let mut sources = ConcatSource::default();

  fragments.iter_mut().for_each(|f| {
    sources.add(RawSource::from(std::mem::take(&mut f.content)));
  });

  sources.add(source);

  fragments.iter_mut().rev().for_each(|f| {
    if let Some(end_content) = std::mem::take(&mut f.end_content) {
      sources.add(RawSource::from(end_content));
    }
  });

  sources.boxed()
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
