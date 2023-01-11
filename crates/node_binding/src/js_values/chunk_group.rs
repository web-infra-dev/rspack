use crate::js_values::JsChunk;

#[napi(object)]
pub struct JsChunkGroup {
  pub chunks: Vec<JsChunk>,
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
        .map(|k| {
          JsChunk::from(
            compilation.chunk_by_ukey.get(k).unwrap_or_else(|| {
              panic!("Could not find Chunk({k:?}) belong to ChunkGroup: {cg:?}",)
            }),
          )
        })
        .collect(),
    }
  }
}
