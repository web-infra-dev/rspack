use rspack_cacheable::{cacheable, with::AsPreset};
use rspack_util::atom::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct LocalModule {
  #[cacheable(with=AsPreset)]
  name: Atom,
  idx: usize,
  used: bool,
  amd_dep_idx: usize,
}

impl LocalModule {
  pub fn new(name: Atom, idx: usize, amd_dep_idx: usize) -> Self {
    Self {
      name,
      idx,
      used: false,
      amd_dep_idx,
    }
  }

  pub fn flag_used(&mut self) {
    self.used = true;
  }

  pub fn variable_name(&self) -> String {
    format!("__rspack_amd_local_{}", self.idx)
  }

  pub fn is_used(&self) -> bool {
    self.used
  }

  pub fn get_name(&self) -> &Atom {
    &self.name
  }

  pub fn amd_dep_idx(&self) -> usize {
    self.amd_dep_idx
  }
}
