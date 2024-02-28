use napi::Result;
use napi_derive::napi;
use rspack_core::{ChunkUkey, SourceType};

use crate::{JsChunk, JsCompilation, JsModule, ToJsModule};

#[napi(js_name = "__chunk_graph_inner_get_chunk_modules")]
pub fn get_chunk_modules(js_chunk_ukey: u32, compilation: &JsCompilation) -> Vec<JsModule> {
  let compilation = &compilation.inner;
  let modules = compilation.chunk_graph.get_chunk_modules(
    &ChunkUkey::from(js_chunk_ukey as usize),
    &compilation.module_graph,
  );

  return modules
    .iter()
    .filter_map(|module| module.to_js_module().ok())
    .collect::<Vec<_>>();
}

#[napi(js_name = "__chunk_graph_inner_get_chunk_entry_modules")]
pub fn get_chunk_entry_modules(js_chunk_ukey: u32, compilation: &JsCompilation) -> Vec<JsModule> {
  let compilation = &compilation.inner;
  let modules = compilation
    .chunk_graph
    .get_chunk_entry_modules(&ChunkUkey::from(js_chunk_ukey as usize));

  return modules
    .iter()
    .filter_map(|module| compilation.module_graph.module_by_identifier(module))
    .filter_map(|module| module.to_js_module().ok())
    .collect::<Vec<_>>();
}

#[napi(js_name = "__chunk_graph_inner_get_chunk_entry_dependent_chunks_iterable")]
pub fn get_chunk_entry_dependent_chunks_iterable(
  js_chunk_ukey: u32,
  compilation: &JsCompilation,
) -> Vec<JsChunk> {
  let compilation = &compilation.inner;
  let chunks = compilation
    .chunk_graph
    .get_chunk_entry_dependent_chunks_iterable(
      &ChunkUkey::from(js_chunk_ukey as usize),
      &compilation.chunk_by_ukey,
      &compilation.chunk_group_by_ukey,
    );

  return chunks
    .into_iter()
    .map(|c| JsChunk::from(compilation.chunk_by_ukey.expect_get(&c)))
    .collect::<Vec<_>>();
}

#[napi(js_name = "__chunk_graph_inner_get_chunk_modules_iterable_by_source_type")]
pub fn get_chunk_modules_iterable_by_source_type(
  js_chunk_ukey: u32,
  source_type: String,
  compilation: &JsCompilation,
) -> Result<Vec<JsModule>> {
  let compilation = &compilation.inner;
  Ok(
    compilation
      .chunk_graph
      .get_chunk_modules_iterable_by_source_type(
        &ChunkUkey::from(js_chunk_ukey as usize),
        SourceType::from(source_type.as_str()),
        &compilation.module_graph,
      )
      .filter_map(|module| module.to_js_module().ok())
      .collect(),
  )
}
