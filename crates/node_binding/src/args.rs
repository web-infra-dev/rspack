use crate::js_values::JsChunk;

#[napi(object)]
pub struct ChunkAssetArgs {
  pub chunk: JsChunk,
  pub filename: String,
}
