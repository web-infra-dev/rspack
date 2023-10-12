use napi_derive::napi;
use rspack_core::ChunkAssetArgs;

#[napi(object)]
pub struct JsChunk {
  pub name: Option<String>,
  pub id: Option<String>,
  pub ids: Vec<String>,
  pub files: Vec<String>,
  pub id_name_hints: Vec<String>,
  pub filename_template: Option<String>,
  pub css_filename_template: Option<String>,
  pub runtime: Vec<String>,
  pub hash: Option<String>,
  pub content_hash: std::collections::HashMap<String, String>,
  pub rendered_hash: Option<String>,
  pub chunk_reasons: Vec<String>,
}

impl JsChunk {
  pub fn from(chunk: &rspack_core::Chunk) -> Self {
    let name = chunk.name.clone();
    let mut files = Vec::from_iter(chunk.files.iter().cloned());
    files.sort_unstable();
    Self {
      name,
      id: chunk.id.clone(),
      ids: chunk.ids.clone(),
      id_name_hints: Vec::from_iter(chunk.id_name_hints.clone().into_iter()),
      filename_template: chunk
        .filename_template
        .as_ref()
        .map(|tpl| tpl.template().to_string()),
      css_filename_template: chunk
        .css_filename_template
        .as_ref()
        .map(|tpl| tpl.template().to_string()),
      files,
      runtime: Vec::from_iter(chunk.runtime.clone().into_iter().map(|r| r.to_string())),
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
