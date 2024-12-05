use std::collections::HashMap;

use napi_derive::napi;
use rspack_collections::DatabaseItem;
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
  pub chunk_reason: Option<String>,
  pub auxiliary_files: Vec<String>,
}

impl JsChunk {
  pub fn from(chunk: &rspack_core::Chunk, compilation: &Compilation) -> Self {
    let mut files = Vec::from_iter(chunk.files().iter().map(ToString::to_string));
    files.sort_unstable();
    let mut auxiliary_files =
      Vec::from_iter(chunk.auxiliary_files().iter().map(ToString::to_string));
    auxiliary_files.sort_unstable();

    Self {
      inner_ukey: chunk.ukey().as_u32(),
      inner_groups: chunk.groups().iter().map(|ukey| ukey.as_u32()).collect(),
      name: chunk.name().map(ToOwned::to_owned),
      id: chunk.id().map(ToOwned::to_owned),
      ids: chunk.id().map_or(Vec::new(), |id| vec![id.to_owned()]),
      id_name_hints: Vec::from_iter(chunk.id_name_hints().clone()),
      filename_template: chunk
        .filename_template()
        .and_then(|f| Some(f.template()?.to_string())),
      css_filename_template: chunk
        .css_filename_template()
        .and_then(|f| Some(f.template()?.to_string())),
      files,
      auxiliary_files,
      runtime: chunk.runtime().iter().map(|r| r.to_string()).collect(),
      hash: chunk
        .hash(&compilation.chunk_hashes_results)
        .map(|d| d.encoded().to_string()),
      content_hash: chunk
        .content_hash(&compilation.chunk_hashes_results)
        .map(|content_hash| {
          content_hash
            .iter()
            .map(|(key, v)| (key.to_string(), v.encoded().to_string()))
            .collect::<std::collections::HashMap<String, String>>()
        })
        .unwrap_or_default(),
      rendered_hash: chunk
        .rendered_hash(
          &compilation.chunk_hashes_results,
          compilation.options.output.hash_digest_length,
        )
        .map(|hash| hash.to_string()),
      chunk_reason: chunk.chunk_reason().map(ToOwned::to_owned),
    }
  }
}

fn chunk(ukey: u32, compilation: &Compilation) -> &Chunk {
  let ukey = ChunkUkey::from(ukey);
  compilation.chunk_by_ukey.expect_get(&ukey)
}

#[napi(js_name = "__chunk_inner_is_only_initial")]
pub fn is_only_initial(js_chunk_ukey: u32, js_compilation: &JsCompilation) -> bool {
  let compilation = unsafe { js_compilation.inner.as_ref() };

  let chunk = chunk(js_chunk_ukey, compilation);
  chunk.is_only_initial(&compilation.chunk_group_by_ukey)
}

#[napi(js_name = "__chunk_inner_can_be_initial")]
pub fn can_be_initial(js_chunk_ukey: u32, js_compilation: &JsCompilation) -> bool {
  let compilation = unsafe { js_compilation.inner.as_ref() };

  let chunk = chunk(js_chunk_ukey, compilation);
  chunk.can_be_initial(&compilation.chunk_group_by_ukey)
}

#[napi(js_name = "__chunk_inner_has_runtime")]
pub fn has_runtime(js_chunk_ukey: u32, js_compilation: &JsCompilation) -> bool {
  let compilation = unsafe { js_compilation.inner.as_ref() };

  let chunk = chunk(js_chunk_ukey, compilation);
  chunk.has_runtime(&compilation.chunk_group_by_ukey)
}

#[napi(js_name = "__chunk_inner_get_all_async_chunks")]
pub fn get_all_async_chunks(js_chunk_ukey: u32, js_compilation: &JsCompilation) -> Vec<JsChunk> {
  let compilation = unsafe { js_compilation.inner.as_ref() };

  let chunk = chunk(js_chunk_ukey, compilation);
  chunk
    .get_all_async_chunks(&compilation.chunk_group_by_ukey)
    .into_iter()
    .map(|c| JsChunk::from(compilation.chunk_by_ukey.expect_get(&c), compilation))
    .collect()
}

#[napi(js_name = "__chunk_inner_get_all_initial_chunks")]
pub fn get_all_initial_chunks(js_chunk_ukey: u32, js_compilation: &JsCompilation) -> Vec<JsChunk> {
  let compilation = unsafe { js_compilation.inner.as_ref() };

  let chunk = chunk(js_chunk_ukey, compilation);
  chunk
    .get_all_initial_chunks(&compilation.chunk_group_by_ukey)
    .into_iter()
    .map(|c| JsChunk::from(compilation.chunk_by_ukey.expect_get(&c), compilation))
    .collect()
}

#[napi(js_name = "__chunk_inner_get_all_referenced_chunks")]
pub fn get_all_referenced_chunks(
  js_chunk_ukey: u32,
  js_compilation: &JsCompilation,
) -> Vec<JsChunk> {
  let compilation = unsafe { js_compilation.inner.as_ref() };

  let chunk = chunk(js_chunk_ukey, compilation);
  chunk
    .get_all_referenced_chunks(&compilation.chunk_group_by_ukey)
    .into_iter()
    .map(|c| JsChunk::from(compilation.chunk_by_ukey.expect_get(&c), compilation))
    .collect()
}

#[napi(object)]
pub struct JsChunkAssetArgs {
  pub chunk: JsChunk,
  pub filename: String,
}
