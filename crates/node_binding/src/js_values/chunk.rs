#[napi(object)]
pub struct JsChunk {
  pub files: Vec<String>,
}

impl JsChunk {
  pub fn from(chunk: &rspack_core::Chunk) -> Self {
    Self {
      files: Vec::from_iter(chunk.files.iter().cloned()),
    }
  }

  pub fn get_files(&self) -> Vec<String> {
    self.files.clone()
  }
}
