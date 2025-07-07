#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct AvailableModules {
  available_modules: num_bigint::BigUint,
}

impl AvailableModules {
  pub fn union(&self, other: &Self) -> Self {
    Self {
      available_modules: &self.available_modules | &other.available_modules,
    }
  }

  fn is_subset_of(&self, other: &Self) -> bool {
    self.intersect(other) == *self
  }

  fn intersect(&self, other: &Self) -> Self {
    Self {
      available_modules: &self.available_modules & &other.available_modules,
    }
  }

  pub fn is_module_available(&self, module: u64) -> bool {
    self.available_modules.bit(module)
  }

  pub fn add(&mut self, module: u64) {
    self.available_modules.set_bit(module, true);
  }

  pub fn should_invalidate(&self, is_entry: bool, curr: &AvailableModules) -> bool {
    if is_entry {
      // for entry chunk, if prev is subset of curr
      // that means we've discoverd more parent chunk
      // current chunk should skip more available modules
      true
    } else {
      // for normal chunk, if curr is subset of prev,
      // that means we've discovered more incomings
      // we should invalidate previous result
      curr.is_subset_of(self)
    }
  }

  pub fn merge_available_modules(&mut self, is_entry: bool, other: &AvailableModules) {
    if is_entry {
      *self = self.union(other);
    } else {
      *self = self.intersect(other);
    }
  }
}
