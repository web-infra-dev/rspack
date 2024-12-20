mod mutations;

use bitflags::bitflags;
pub use mutations::{Mutation, Mutations};

bitflags! {
  #[derive(Debug, Clone, Copy, Eq, PartialEq)]
  pub struct IncrementalPasses: u16 {
    const MAKE = 1 << 0;
    const INFER_ASYNC_MODULES = 1 << 1;
    const PROVIDED_EXPORTS = 1 << 2;
    const DEPENDENCIES_DIAGNOSTICS = 1 << 3;
    const SIDE_EFFECTS = 1 << 4;
    const BUILD_CHUNK_GRAPH = 1 << 5;
    const MODULE_IDS = 1 << 6;
    const CHUNK_IDS = 1 << 7;
    const MODULES_HASHES = 1 << 8;
    const MODULES_CODEGEN = 1 << 9;
    const MODULES_RUNTIME_REQUIREMENTS = 1 << 10;
    const CHUNKS_RUNTIME_REQUIREMENTS = 1 << 11;
    const CHUNKS_HASHES = 1 << 12;
    const CHUNKS_RENDER = 1 << 13;
    const EMIT_ASSETS = 1 << 14;
  }
}

#[derive(Debug)]
pub struct Incremental {
  passes: IncrementalPasses,
  mutations: Mutations,
}

impl Incremental {
  pub fn new(passes: IncrementalPasses) -> Self {
    Self {
      passes,
      mutations: Mutations::default(),
    }
  }

  pub fn can_write_mutations(&self) -> bool {
    !self.passes.is_empty()
  }

  pub fn can_read_mutations(&self, pass: IncrementalPasses) -> bool {
    self.passes.contains(pass)
  }

  pub fn mutations_write(&mut self) -> Option<&mut Mutations> {
    self.can_write_mutations().then_some(&mut self.mutations)
  }

  pub fn mutations_read(&self, pass: IncrementalPasses) -> Option<&Mutations> {
    self.can_read_mutations(pass).then_some(&self.mutations)
  }
}
