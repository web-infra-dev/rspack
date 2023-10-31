use napi_derive::napi;
use rspack_core::ChunkAssetArgs;

#[napi(object)]
pub struct JsChunk {
  pub name: Option<String>,
  pub files: Vec<String>,
}

impl JsChunk {
  pub fn from(chunk: &rspack_core::Chunk) -> Self {
    let name = chunk.name.clone();
    let mut files = Vec::from_iter(chunk.files.iter().cloned());
    files.sort_unstable();
    Self { name, files }
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
