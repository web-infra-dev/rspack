use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::{Chunk, ChunkAssetArgs, ChunkUkey, Compilation};

use crate::JsCompilation;

#[napi(object)]
pub struct JsChunk {
  #[napi(js_name = "__inner_ukey")]
  pub inner_ukey: u32, // ChunkUkey
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
}

impl JsChunk {
  pub fn from(chunk: &rspack_core::Chunk) -> Self {
    let name = chunk.name.clone();
    let mut files = Vec::from_iter(chunk.files.iter().cloned());
    files.sort_unstable();

    Self {
      inner_ukey: usize::from(chunk.ukey) as u32,
      name,
      id: chunk.id.clone(),
      ids: chunk.ids.clone(),
      id_name_hints: Vec::from_iter(chunk.id_name_hints.clone()),
      filename_template: chunk
        .filename_template
        .as_ref()
        .map(|tpl| tpl.template().to_string()),
      css_filename_template: chunk
        .css_filename_template
        .as_ref()
        .map(|tpl| tpl.template().to_string()),
      files,
      runtime: Vec::<String>::from_iter(chunk.runtime.clone().into_iter().map(|r| r.to_string())),
      hash: chunk.hash.as_ref().map(|d| d.encoded().to_string()),
      content_hash: chunk
        .content_hash
        .iter()
        .map(|(key, v)| (key.to_string(), v.encoded().to_string()))
        .collect::<std::collections::HashMap<String, String>>(),
      rendered_hash: chunk.rendered_hash.as_ref().map(|hash| hash.to_string()),
      chunk_reasons: chunk.chunk_reasons.clone(),
    }
  }

  fn chunk<'compilation>(&self, compilation: &'compilation Compilation) -> &'compilation Chunk {
    let inner_key = self.inner_ukey;
    let ukey = ChunkUkey::from(inner_key as usize);

    compilation
      .chunk_by_ukey
      .get(&ukey)
      .expect("Chunk must exist")
  }
}

#[napi(js_name = "__chunk_inner_is_only_initial")]
pub fn is_only_initial(js_chunk: JsChunk, compilation: &JsCompilation) -> bool {
  let compilation = &compilation.inner;
  let chunk = js_chunk.chunk(compilation);
  chunk.is_only_initial(&compilation.chunk_group_by_ukey)
}

#[napi(js_name = "__chunk_inner_can_be_initial")]
pub fn can_be_initial(js_chunk: JsChunk, compilation: &JsCompilation) -> bool {
  let compilation = &compilation.inner;
  let chunk = js_chunk.chunk(compilation);
  chunk.can_be_initial(&compilation.chunk_group_by_ukey)
}

#[napi(js_name = "__chunk_inner_has_runtime")]
pub fn has_runtime(js_chunk: JsChunk, compilation: &JsCompilation) -> bool {
  let compilation = &compilation.inner;
  let chunk = js_chunk.chunk(compilation);
  chunk.has_runtime(&compilation.chunk_group_by_ukey)
}

#[napi(object)]
pub struct JsChunkAssetArgs {
  pub chunk: JsChunk,
  pub filename: String,
}

impl From<&ChunkAssetArgs<'_>> for JsChunkAssetArgs {
  fn from(value: &ChunkAssetArgs) -> Self {
    Self {
      chunk: JsChunk::from(value.chunk),
      filename: value.filename.to_string(),
    }
  }
}
