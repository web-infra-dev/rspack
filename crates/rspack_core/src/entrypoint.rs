use std::ops::{Deref, DerefMut};

use crate::ChunkGroup;

#[derive(Debug, Default)]
pub struct Entrypoint {
  // pub(crate) files: Vec<String>,
  chunk_group: ChunkGroup,
}

// The implmentation is temporary and will be aligned with Webpack.
impl Entrypoint {
  pub fn new() -> Self {
    Self {
      // files,
      chunk_group: Default::default(),
    }
  }
}

// Use `Deref` to stimulate the inheritance in OO language.
// This is not recommended in Rust, but it's helpful for porting.
impl Deref for Entrypoint {
  type Target = ChunkGroup;
  fn deref(&self) -> &Self::Target {
    &self.chunk_group
  }
}

impl DerefMut for Entrypoint {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.chunk_group
  }
}
