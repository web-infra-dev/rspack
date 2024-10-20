use rspack_util::atom::Atom;

#[derive(Debug, Clone)]
pub struct LocalModule {
  name: Atom,
  idx: usize,
  used: bool,
}

impl LocalModule {
  pub fn new(name: Atom, idx: usize) -> Self {
    Self {
      name,
      idx,
      used: false,
    }
  }

  pub fn flag_used(&mut self) {
    self.used = true;
  }

  pub fn variable_name(&self) -> String {
    format!("__WEBPACK_LOCAL_MODULE_{}__", self.idx)
  }

  pub fn is_used(&self) -> bool {
    self.used
  }

  pub fn get_name(&self) -> &Atom {
    &self.name
  }
}
