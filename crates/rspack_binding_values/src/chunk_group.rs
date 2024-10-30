use napi::bindgen_prelude::FromNapiValue;
use napi_derive::napi;
use rspack_core::{ChunkGroup, ChunkGroupUkey, Compilation};

use crate::{JsChunk, JsCompilation, JsModuleWrapper};

#[napi(object)]
pub struct JsChunkGroup {
  #[napi(js_name = "__inner_parents")]
  pub inner_parents: Vec<u32>,
  #[napi(js_name = "__inner_ukey")]
  pub inner_ukey: u32,
  pub chunks: Vec<JsChunk>,
  pub index: Option<u32>,
  pub name: Option<String>,
  pub is_initial: bool,
  pub origins: Vec<JsChunkGroupOrigin>,
}

#[napi(object, object_from_js = false)]
pub struct JsChunkGroupOrigin {
  #[napi(ts_type = "JsModule | undefined")]
  pub module: Option<JsModuleWrapper>,
  pub request: Option<String>,
}

impl FromNapiValue for JsChunkGroupOrigin {
  unsafe fn from_napi_value(
    _env: napi::sys::napi_env,
    _napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    unreachable!()
  }
}

impl JsChunkGroup {
  pub fn from_chunk_group(
    cg: &rspack_core::ChunkGroup,
    compilation: &rspack_core::Compilation,
  ) -> Self {
    Self {
      chunks: cg
        .chunks
        .iter()
        .map(|k| JsChunk::from(compilation.chunk_by_ukey.expect_get(k)))
        .collect(),
      index: cg.index,
      inner_parents: cg.parents.iter().map(|ukey| ukey.as_u32()).collect(),
      inner_ukey: cg.ukey.as_u32(),
      name: cg.name().map(|name| name.to_string()),
      is_initial: cg.is_initial(),
      origins: cg
        .origins()
        .iter()
        .map(|origin| JsChunkGroupOrigin {
          module: origin.module_id.map(|module_id| {
            let module = compilation
              .module_by_identifier(&module_id)
              .unwrap_or_else(|| panic!("failed to retrieve module by id: {}", module_id));
            JsModuleWrapper::new(module.as_ref(), Some(compilation))
          }),
          request: origin.request.clone(),
        })
        .collect::<Vec<_>>(),
    }
  }
}

fn chunk_group(ukey: u32, compilation: &Compilation) -> &ChunkGroup {
  let ukey = ChunkGroupUkey::from(ukey);
  compilation.chunk_group_by_ukey.expect_get(&ukey)
}

#[napi(js_name = "__chunk_group_inner_get_chunk_group")]
pub fn get_chunk_group(ukey: u32, js_compilation: &JsCompilation) -> JsChunkGroup {
  let compilation = unsafe { js_compilation.0.as_ref() };

  let cg = chunk_group(ukey, compilation);
  JsChunkGroup::from_chunk_group(cg, compilation)
}

#[napi(js_name = "__entrypoint_inner_get_runtime_chunk")]
pub fn get_runtime_chunk(ukey: u32, js_compilation: &JsCompilation) -> JsChunk {
  let compilation = unsafe { js_compilation.0.as_ref() };

  let entrypoint = chunk_group(ukey, compilation);
  let chunk_ukey = entrypoint.get_runtime_chunk(&compilation.chunk_group_by_ukey);
  let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
  JsChunk::from(chunk)
}
