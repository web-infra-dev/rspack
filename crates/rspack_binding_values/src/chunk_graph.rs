use std::ptr::NonNull;

use napi_derive::napi;
use rspack_core::{ChunkUkey, Compilation, SourceType};
use rspack_napi::napi::Result;

use crate::{JsChunk, JsCompilation, JsModuleWrapper};

#[napi]
pub struct JsChunkGraph {
  compilation: NonNull<Compilation>,
}

impl JsChunkGraph {
  pub fn new(compilation: &Compilation) -> Self {
    #[allow(clippy::unwrap_used)]
    JsChunkGraph {
      compilation: NonNull::new(compilation as *const Compilation as *mut Compilation).unwrap(),
    }
  }

  fn as_ref(&self) -> napi::Result<&'static Compilation> {
    let compilation = unsafe { self.compilation.as_ref() };
    Ok(compilation)
  }
}

#[napi]
impl JsChunkGraph {
  #[napi(ts_return_type = "JsModule[]")]
  pub fn get_chunk_modules(&self, js_chunk_ukey: u32) -> Vec<JsModuleWrapper> {
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
