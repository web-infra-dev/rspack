use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::{Chunk, ChunkAssetArgs, ChunkUkey, Compilation};

#[napi(object)]
pub struct JsChunk {
  pub inner: u32,
}

#[napi(object)]
pub struct JsChunkStruct {
  pub name: Option<String>,
  pub filename_template: Option<String>,
  pub css_filename_template: Option<String>,
  pub id: Option<String>,
  pub ids: Vec<String>,
  pub id_name_hints: Vec<String>,
  pub files: Vec<String>,
  pub auxiliary_files: Vec<String>,
  pub runtime: Vec<String>,
  pub hash: Option<String>,
  pub rendered_hash: Option<String>,
  pub content_hash: HashMap<String, String>,
  pub chunk_reasons: Vec<String>,
}

impl JsChunk {
  pub fn from(chunk: &rspack_core::Chunk) -> Self {
    Self {
      inner: chunk.ukey.as_usize() as u32,
    }
    // let name = chunk.name.clone();
    // let mut files = Vec::from_iter(chunk.files.iter().cloned());
    // files.sort_unstable();
    // Self {
    //   name,
    //   id: chunk.id.clone(),
    //   ids: chunk.ids.clone(),
    //   id_name_hints: Vec::from_iter(chunk.id_name_hints.clone().into_iter()),
    //   filename_template: chunk
    //     .filename_template
    //     .as_ref()
    //     .map(|tpl| tpl.template().to_string()),
    //   css_filename_template: chunk
    //     .css_filename_template
    //     .as_ref()
    //     .map(|tpl| tpl.template().to_string()),
    //   files,
    //   runtime: Vec::from_iter(chunk.runtime.clone().into_iter().map(|r| r.to_string())),
    //   hash: chunk.hash.as_ref().map(|d| d.encoded().to_string()),
    //   content_hash: chunk
    //     .content_hash
    //     .iter()
    //     .map(|(key, v)| (key.to_string(), v.encoded().to_string()))
    //     .collect::<std::collections::HashMap<String, String>>(),
    //   rendered_hash: chunk.rendered_hash.as_ref().map(|hash| hash.to_string()),
    //   chunk_reasons: chunk.chunk_reasons.clone(),
    // }
  }

  fn chunk<'compilation>(&self, compilation: &'compilation Compilation) -> &'compilation Chunk {
    let ukey = ChunkUkey::from(self.inner as usize);

    compilation
      .chunk_by_ukey
      .get(&ukey)
      .expect("Chunk must exist")
  }
}

#[napi]
impl JsChunk {
  // TODO: we should support more universal way to support efficient field access
  pub fn get_struct(&self, compilation: &Compilation) -> JsChunkStruct {
    let chunk = self.chunk(compilation);
    JsChunkStruct {
      name: chunk.name.clone(),
      filename_template: chunk
        .filename_template
        .as_ref()
        .map(|t| t.template().to_string()),
      css_filename_template: chunk
        .css_filename_template
        .as_ref()
        .map(|t| t.template().to_string()),
      id: chunk.id.clone(),
      ids: chunk.ids.clone(),
      id_name_hints: Vec::from_iter(chunk.id_name_hints.clone().into_iter()),
      files: Vec::from_iter(chunk.files.clone().into_iter()),
      auxiliary_files: Vec::from_iter(chunk.auxiliary_files.clone().into_iter()),
      runtime: Vec::from_iter(
        chunk
          .runtime
          .clone()
          .into_iter()
          .map(|runtime| runtime.to_string()),
      ),
      hash: chunk.hash.as_ref().map(|hash| hash.encoded().into()),
      rendered_hash: chunk.rendered_hash.as_ref().map(|hash| hash.to_string()),
      content_hash: HashMap::from_iter(
        chunk
          .content_hash
          .iter()
          .map(|(k, v)| (k.to_string(), v.encoded().to_string())),
      ),
      chunk_reasons: chunk.chunk_reasons.clone(),
    }
  }

  pub fn is_only_initial(&self, compilation: &Compilation) -> bool {
    let chunk = self.chunk(compilation);
    chunk.is_only_initial(&compilation.chunk_group_by_ukey)
  }

  pub fn can_be_initial(&self, compilation: &Compilation) -> bool {
    let chunk = self.chunk(compilation);
    chunk.can_be_initial(&compilation.chunk_group_by_ukey)
  }

  pub fn has_runtime(&self, compilation: &Compilation) -> bool {
    let chunk = self.chunk(compilation);
    chunk.has_runtime(&compilation.chunk_group_by_ukey)
  }
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
