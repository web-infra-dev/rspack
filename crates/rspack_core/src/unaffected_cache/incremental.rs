use bitflags::bitflags;

use super::Mutations;

bitflags! {
  #[derive(Debug, Clone, Copy, Eq, PartialEq)]
  pub struct IncrementalPasses: u8 {
    const MAKE = 1 << 0;
    const INFER_ASYNC_MODULES = 1 << 1;
    const PROVIDED_EXPORTS = 1 << 2;
    const COLLECT_MODULES_DIAGNOSTICS = 1 << 3;
    const MODULE_HASHES = 1 << 4;
    const MODULE_CODEGEN = 1 << 5;
    const MODULE_RUNTIME_REQUIREMENTS = 1 << 6;
    const EMIT_ASSETS = 1 << 7;
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
