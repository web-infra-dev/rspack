use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::{Chunk, ChunkUkey, Compilation};

use crate::JsCompilation;

#[napi(object)]
pub struct JsChunk {
  #[napi(js_name = "__inner_ukey")]
  pub inner_ukey: u32, // ChunkUkey
  #[napi(js_name = "__inner_groups")]
  pub inner_groups: Vec<u32>,
  pub name: Option<String>,
  pub id: Option<String>,
  pub ids: Vec<String>,
  pub id_name_hints: Vec<String>,
  pub filename_template: Option<String>,
  pub css_filename_template: Option<String>,
  pub files: Vec<String>,
  pub runtime: Vec<String>,
  pub hash: Option<String>,
  pub content_hash: HashMap<String, String>,
  pub rendered_hash: Option<String>,
  pub chunk_reasons: Vec<String>,
  pub auxiliary_files: Vec<String>,
}

impl JsChunk {
  pub fn from(chunk: &rspack_core::Chunk) -> Self {
    let Chunk {
      // not implement yet
      ukey: _ukey,
      prevent_integration: _prevent_integration,
      groups: _groups,
      kind: _kind,

      // used in js chunk
      name,
      filename_template,
      css_filename_template,
      id,
      ids,
      id_name_hints,
      files,
      auxiliary_files,
      runtime,
      hash,
      rendered_hash,
      content_hash,
      chunk_reasons,
    } = chunk;
    let mut files = Vec::from_iter(files.iter().cloned());
    files.sort_unstable();
    let mut auxiliary_files = auxiliary_files.iter().cloned().collect::<Vec<_>>();
    auxiliary_files.sort_unstable();
    let mut runtime = Vec::<String>::from_iter(runtime.clone().into_iter().map(|r| r.to_string()));
    runtime.sort_unstable();

    Self {
      inner_ukey: usize::from(chunk.ukey) as u32,
      inner_groups: chunk
        .groups
        .iter()
        .map(|ukey| ukey.as_usize() as u32)
        .collect(),
      name: name.clone(),
      id: id.clone(),
      ids: ids.clone(),
      id_name_hints: Vec::from_iter(id_name_hints.clone()),
      filename_template: filename_template
        .as_ref()
        .map(|tpl| tpl.template().to_string()),
      css_filename_template: css_filename_template
        .as_ref()
        .map(|tpl| tpl.template().to_string()),
      files,
      runtime,
      hash: hash.as_ref().map(|d| d.encoded().to_string()),
      content_hash: content_hash
        .iter()
        .map(|(key, v)| (key.to_string(), v.encoded().to_string()))
        .collect::<std::collections::HashMap<String, String>>(),
      rendered_hash: rendered_hash.as_ref().map(|hash| hash.to_string()),
      chunk_reasons: chunk_reasons.clone(),
      auxiliary_files,
    }
  }
}

fn chunk(ukey: u32, compilation: &Compilation) -> &Chunk {
  let ukey = ChunkUkey::from(ukey as usize);
  compilation.chunk_by_ukey.expect_get(&ukey)
}

#[napi(js_name = "__chunk_inner_is_only_initial")]
pub fn is_only_initial(js_chunk_ukey: u32, compilation: &JsCompilation) -> bool {
  let compilation = &compilation.0;
  let chunk = chunk(js_chunk_ukey, compilation);
  chunk.is_only_initial(&compilation.chunk_group_by_ukey)
}

#[napi(js_name = "__chunk_inner_can_be_initial")]
pub fn can_be_initial(js_chunk_ukey: u32, compilation: &JsCompilation) -> bool {
  let compilation = &compilation.0;
  let chunk = chunk(js_chunk_ukey, compilation);
  chunk.can_be_initial(&compilation.chunk_group_by_ukey)
}

#[napi(js_name = "__chunk_inner_has_runtime")]
pub fn has_runtime(js_chunk_ukey: u32, compilation: &JsCompilation) -> bool {
  let compilation = &compilation.0;
  let chunk = chunk(js_chunk_ukey, compilation);
  chunk.has_runtime(&compilation.chunk_group_by_ukey)
}

#[napi(js_name = "__chunk_inner_get_all_async_chunks")]
pub fn get_all_async_chunks(js_chunk_ukey: u32, compilation: &JsCompilation) -> Vec<JsChunk> {
  let compilation = &compilation.0;
  let chunk = chunk(js_chunk_ukey, compilation);
  chunk
    .get_all_async_chunks(&compilation.chunk_group_by_ukey)
    .into_iter()
    .map(|c| JsChunk::from(compilation.chunk_by_ukey.expect_get(&c)))
    .collect()
}

#[napi(js_name = "__chunk_inner_get_all_initial_chunks")]
pub fn get_all_initial_chunks(js_chunk_ukey: u32, compilation: &JsCompilation) -> Vec<JsChunk> {
  let compilation = &compilation.0;
  let chunk = chunk(js_chunk_ukey, compilation);
  chunk
    .get_all_initial_chunks(&compilation.chunk_group_by_ukey)
    .into_iter()
    .map(|c| JsChunk::from(compilation.chunk_by_ukey.expect_get(&c)))
    .collect()
}

#[napi(js_name = "__chunk_inner_get_all_referenced_chunks")]
pub fn get_all_referenced_chunks(js_chunk_ukey: u32, compilation: &JsCompilation) -> Vec<JsChunk> {
  let compilation = &compilation.0;
  let chunk = chunk(js_chunk_ukey, compilation);
  chunk
    .get_all_referenced_chunks(&compilation.chunk_group_by_ukey)
    .into_iter()
    .map(|c| JsChunk::from(compilation.chunk_by_ukey.expect_get(&c)))
    .collect()
}

#[napi(object)]
pub struct JsChunkAssetArgs {
  pub chunk: JsChunk,
  pub filename: String,
}
