#[derive(Debug)]
pub struct CodeSplittingOptions {
  pub reuse_existing_chunk: bool,
}

impl Default for CodeSplittingOptions {
  fn default() -> Self {
    Self {
      reuse_existing_chunk: true,
    }
  }
}
