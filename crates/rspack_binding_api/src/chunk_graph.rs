use std::ptr::NonNull;

use napi::{Either, Result};
use napi_derive::napi;
use rspack_core::{Compilation, ModuleId, SourceType};

use crate::{
  async_dependency_block::AsyncDependenciesBlock,
  chunk::{Chunk, ChunkWrapper},
  chunk_group::ChunkGroupWrapper,
  module::{ModuleObject, ModuleObjectRef},
  runtime::JsRuntimeSpec,
};

pub type JsModuleId = Either<String, u32>;

pub fn to_js_module_id(module_id: &ModuleId) -> JsModuleId {
  if let Some(id) = module_id.as_number() {
    JsModuleId::B(id)
  } else {
    JsModuleId::A(module_id.to_string())
  }
}

#[napi]
pub struct ChunkGraph {
  compilation: NonNull<Compilation>,
}

impl ChunkGraph {
  pub fn new(compilation: &Compilation) -> Self {
    #[allow(clippy::unwrap_used)]
    ChunkGraph {
      compilation: NonNull::new(compilation as *const Compilation as *mut Compilation).unwrap(),
    }
  }

  fn as_ref(&self) -> Result<&'static Compilation> {
    let compilation = unsafe { self.compilation.as_ref() };
    Ok(compilation)
  }
}

#[napi]
impl ChunkGraph {
  #[napi(ts_return_type = "boolean")]
  pub fn has_chunk_entry_dependent_chunks(&self, chunk: &Chunk) -> Result<bool> {
    let compilation = self.as_ref()?;
    Ok(
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .has_chunk_entry_dependent_chunks(
          &chunk.chunk_ukey,
          &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
        ),
    )
  }

  #[napi(ts_return_type = "Module[]")]
  pub fn get_chunk_modules(&self, chunk: &Chunk) -> Result<Vec<ModuleObject>> {
    let compilation = self.as_ref()?;

    let module_graph = compilation.get_module_graph();
    let modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_modules(&chunk.chunk_ukey, module_graph);

    Ok(
      modules
        .iter()
        .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id()))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "Iterable<Module>")]
  pub fn get_chunk_modules_iterable(&self, chunk: &Chunk) -> Result<Vec<ModuleObject>> {
    self.get_chunk_modules(chunk)
  }

  #[napi(ts_return_type = "Iterable<Module>")]
  pub fn get_chunk_entry_modules_iterable(&self, chunk: &Chunk) -> Result<Vec<ModuleObject>> {
    let compilation = self.as_ref()?;

    let modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_entry_modules(&chunk.chunk_ukey);
    let module_graph = compilation.get_module_graph();
    Ok(
      modules
        .iter()
        .filter_map(|module| module_graph.module_by_identifier(module))
        .map(|module| ModuleObject::with_ref(module.as_ref(), compilation.compiler_id()))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "number")]
  pub fn get_number_of_entry_modules(&self, chunk: &Chunk) -> Result<u32> {
    let compilation = self.as_ref()?;

    Ok(
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_number_of_entry_modules(&chunk.chunk_ukey) as u32,
    )
  }

  #[napi(ts_return_type = "Chunk[]")]
  pub fn get_chunk_entry_dependent_chunks_iterable(
    &self,
    chunk: &Chunk,
  ) -> Result<Vec<ChunkWrapper>> {
    let compilation = self.as_ref()?;

    let chunks = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_entry_dependent_chunks_iterable(
        &chunk.chunk_ukey,
        &compilation.build_chunk_graph_artifact.chunk_by_ukey,
        &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
      );

    Ok(
      chunks
        .into_iter()
        .map(|c| ChunkWrapper::new(c, compilation))
        .collect::<Vec<_>>(),
    )
  }

  #[napi(ts_return_type = "Module[]")]
  pub fn get_chunk_modules_iterable_by_source_type(
    &self,
    chunk: &Chunk,
    source_type: String,
  ) -> Result<Vec<ModuleObject>> {
    let compilation = self.as_ref()?;
    let module_graph = compilation.get_module_graph();

    let chunk_modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_modules_by_source_type(
        &chunk.chunk_ukey,
        SourceType::from(source_type.as_str()),
        module_graph,
      );

    Ok(
      chunk_modules
        .into_iter()
        .map(|module| ModuleObject::with_ref(module, compilation.compiler_id()))
        .collect(),
    )
  }

  #[napi(ts_args_type = "module: Module", ts_return_type = "Chunk[]")]
  pub fn get_module_chunks(&self, module: ModuleObjectRef) -> Result<Vec<ChunkWrapper>> {
    let compilation = self.as_ref()?;

    Ok(
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_module_chunks(module.identifier)
        .iter()
        .map(|chunk| ChunkWrapper::new(*chunk, compilation))
        .collect(),
    )
  }

  #[napi(
    ts_args_type = "module: Module",
    ts_return_type = "string | number | null"
  )]
  pub fn get_module_id(&self, module: ModuleObjectRef) -> napi::Result<Option<JsModuleId>> {
    let compilation = self.as_ref()?;
    Ok(
      rspack_core::ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier)
        .map(to_js_module_id),
    )
  }

  #[napi(
    js_name = "_getModuleHash",
    ts_args_type = "module: Module, runtime: string | string[] | undefined"
  )]
  pub fn get_module_hash(
    &self,
    js_module: ModuleObjectRef,
    js_runtime: JsRuntimeSpec,
  ) -> napi::Result<Option<&str>> {
    let compilation = self.as_ref()?;
    let Some(runtime) = js_runtime.map(|js_runtime| match js_runtime {
      Either::A(str) => std::iter::once(str).map(Into::into).collect(),
      Either::B(vec) => vec.into_iter().map(Into::into).collect(),
    }) else {
      return Ok(None);
    };
    Ok(
      rspack_core::ChunkGraph::get_module_hash(compilation, js_module.identifier, &runtime)
        .map(|hash| hash.encoded()),
    )
  }

  #[napi(ts_return_type = "ChunkGroup | null")]
  pub fn get_block_chunk_group(
    &self,
    js_block: &AsyncDependenciesBlock,
  ) -> napi::Result<Option<ChunkGroupWrapper>> {
    let compilation = self.as_ref()?;
    Ok(
      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_block_chunk_group(
          &js_block.block_id,
          &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
        )
        .map(|chunk_group| ChunkGroupWrapper::new(chunk_group.ukey, compilation)),
    )
  }
}
