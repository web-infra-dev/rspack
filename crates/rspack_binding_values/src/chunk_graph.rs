use napi::{Env, JsString};
use napi_derive::napi;
use rspack_core::{ChunkUkey, SourceType};
use rspack_napi::napi::Result;

use crate::{JsChunk, JsCompilation, JsModule, JsModuleWrapper};

#[napi(
  js_name = "__chunk_graph_inner_get_chunk_modules",
  ts_return_type = "JsModule[]"
)]
pub fn get_chunk_modules(
  js_chunk_ukey: u32,
  js_compilation: &JsCompilation,
) -> Vec<JsModuleWrapper> {
  let compilation = unsafe { js_compilation.inner.as_ref() };

  let module_graph = compilation.get_module_graph();
  let modules = compilation
    .chunk_graph
    .get_chunk_modules(&ChunkUkey::from(js_chunk_ukey), &module_graph);

  return modules
    .iter()
    .map(|module| JsModuleWrapper::new(module.as_ref(), compilation.id(), Some(compilation)))
    .collect::<Vec<_>>();
}

#[napi(
  js_name = "__chunk_graph_inner_get_chunk_entry_modules",
  ts_return_type = "JsModule[]"
)]
pub fn get_chunk_entry_modules(
  js_chunk_ukey: u32,
  js_compilation: &JsCompilation,
) -> Vec<JsModuleWrapper> {
  let compilation = unsafe { js_compilation.inner.as_ref() };

  let modules = compilation
    .chunk_graph
    .get_chunk_entry_modules(&ChunkUkey::from(js_chunk_ukey));
  let module_graph = compilation.get_module_graph();
  return modules
    .iter()
    .filter_map(|module| module_graph.module_by_identifier(module))
    .map(|module| JsModuleWrapper::new(module.as_ref(), compilation.id(), Some(compilation)))
    .collect::<Vec<_>>();
}

#[napi(js_name = "__chunk_graph_inner_get_chunk_entry_dependent_chunks_iterable")]
pub fn get_chunk_entry_dependent_chunks_iterable(
  js_chunk_ukey: u32,
  js_compilation: &JsCompilation,
) -> Vec<JsChunk> {
  let compilation = unsafe { js_compilation.inner.as_ref() };

  let chunks = compilation
    .chunk_graph
    .get_chunk_entry_dependent_chunks_iterable(
      &ChunkUkey::from(js_chunk_ukey),
      &compilation.chunk_by_ukey,
      &compilation.chunk_group_by_ukey,
    );

  return chunks
    .into_iter()
    .map(|c| JsChunk::from(compilation.chunk_by_ukey.expect_get(&c), compilation))
    .collect::<Vec<_>>();
}

#[napi(
  js_name = "__chunk_graph_inner_get_chunk_modules_iterable_by_source_type",
  ts_return_type = "JsModule[]"
)]
pub fn get_chunk_modules_iterable_by_source_type(
  js_chunk_ukey: u32,
  source_type: String,
  js_compilation: &JsCompilation,
) -> Result<Vec<JsModuleWrapper>> {
  let compilation = unsafe { js_compilation.inner.as_ref() };

  Ok(
    compilation
      .chunk_graph
      .get_chunk_modules_iterable_by_source_type(
        &ChunkUkey::from(js_chunk_ukey),
        SourceType::from(source_type.as_str()),
        &compilation.get_module_graph(),
      )
      .map(|module| JsModuleWrapper::new(module, compilation.id(), Some(compilation)))
      .collect(),
  )
}

#[napi(js_name = "__chunk_graph_inner_get_module_id")]
pub fn get_module_id(
  env: Env,
  js_module: &JsModule,
  js_compilation: &JsCompilation,
) -> Result<Option<JsString>> {
  let compilation = unsafe { js_compilation.inner.as_ref() };

  match compilation.chunk_graph.get_module_id(js_module.identifier) {
    Some(id) => {
      let js_value = env.create_string(id)?;
      Ok(Some(js_value))
    }
    None => Ok(None),
  }
}

#[napi(js_name = "__chunk_graph_inner_get_module_chunks")]
pub fn get_module_chunks(js_module: &JsModule, js_compilation: &JsCompilation) -> Vec<JsChunk> {
  let compilation = unsafe { js_compilation.inner.as_ref() };
  let module_chunks = compilation
    .chunk_graph
    .get_module_chunks(js_module.identifier);
  module_chunks
    .iter()
    .filter_map(|chunk_ukey| {
      let chunk = compilation.chunk_by_ukey.get(chunk_ukey);
      chunk.map(|chunk| JsChunk::from(chunk, compilation))
    })
    .collect::<Vec<_>>()
}
