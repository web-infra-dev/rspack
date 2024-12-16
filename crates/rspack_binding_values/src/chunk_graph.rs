use std::ptr::NonNull;

use napi::Result;
use napi_derive::napi;
use rspack_core::{ChunkGraph, Compilation, SourceType};

use crate::{JsChunk, JsChunkWrapper, JsModule, JsModuleWrapper};

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

  fn as_ref(&self) -> Result<&'static Compilation> {
    let compilation = unsafe { self.compilation.as_ref() };
    Ok(compilation)
  }
}

#[napi]
impl JsChunkGraph {
  #[napi(ts_return_type = "JsModule[]")]
  pub fn get_chunk_modules(&self, chunk: &JsChunk) -> Result<Vec<JsModuleWrapper>> {
    let compilation = self.as_ref()?;

    let module_graph = compilation.get_module_graph();
    let modules = compilation
      .chunk_graph
      .get_chunk_modules(&chunk.chunk_ukey, &module_graph);

    Ok(
      modules
        .iter()
        .map(|module| JsModuleWrapper::new(module.as_ref(), compilation.id(), Some(compilation)))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "JsModule[]")]
  pub fn get_chunk_entry_modules(&self, chunk: &JsChunk) -> Result<Vec<JsModuleWrapper>> {
    let compilation = self.as_ref()?;

    let modules = compilation
      .chunk_graph
      .get_chunk_entry_modules(&chunk.chunk_ukey);
    let module_graph = compilation.get_module_graph();
    Ok(
      modules
        .iter()
        .filter_map(|module| module_graph.module_by_identifier(module))
        .map(|module| JsModuleWrapper::new(module.as_ref(), compilation.id(), Some(compilation)))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "JsChunk[]")]
  pub fn get_chunk_entry_dependent_chunks_iterable(
    &self,
    chunk: &JsChunk,
  ) -> Result<Vec<JsChunkWrapper>> {
    let compilation = self.as_ref()?;

    let chunks = compilation
      .chunk_graph
      .get_chunk_entry_dependent_chunks_iterable(
        &chunk.chunk_ukey,
        &compilation.chunk_by_ukey,
        &compilation.chunk_group_by_ukey,
      );

    Ok(
      chunks
        .into_iter()
        .map(|c| JsChunkWrapper::new(c, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "JsModule[]")]
  pub fn get_chunk_modules_iterable_by_source_type(
    &self,
    chunk: &JsChunk,
    source_type: String,
  ) -> Result<Vec<JsModuleWrapper>> {
    let compilation = self.as_ref()?;

    Ok(
      compilation
        .chunk_graph
        .get_chunk_modules_iterable_by_source_type(
          &chunk.chunk_ukey,
          SourceType::from(source_type.as_str()),
          &compilation.get_module_graph(),
        )
        .map(|module| JsModuleWrapper::new(module, compilation.id(), Some(compilation)))
        .collect(),
    )
  }

  #[napi(ts_return_type = "JsChunk[]")]
  pub fn get_module_chunks(&self, module: &JsModule) -> Result<Vec<JsChunkWrapper>> {
    let compilation = self.as_ref()?;

    Ok(
      compilation
        .chunk_graph
        .get_module_chunks(module.identifier)
        .iter()
        .map(|chunk| JsChunkWrapper::new(*chunk, compilation))
        .collect(),
    )
  }

  #[napi]
  pub fn get_module_id(&self, js_module: &JsModule) -> napi::Result<Option<&str>> {
    let compilation = self.as_ref()?;
    Ok(
      ChunkGraph::get_module_id(&compilation.module_ids, js_module.identifier)
        .map(|module_id| module_id.as_str()),
    )
  }
}
