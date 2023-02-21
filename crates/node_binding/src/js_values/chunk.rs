#[napi(object)]
pub struct JsChunk {
  pub files: Vec<String>,
}

impl JsChunk {
  pub fn from(chunk: &rspack_core::Chunk) -> Self {
    let mut files = Vec::from_iter(chunk.files.iter().cloned());
    files.sort_unstable();
    Self { files }
  }
}
