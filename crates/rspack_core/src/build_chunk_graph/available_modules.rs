use std::borrow::Cow;

use indexmap::IndexSet;
use rayon::prelude::*;
use rspack_collections::IdentifierMap;
use rustc_hash::FxHashSet as HashSet;
use tracing::instrument;

use crate::Compilation;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AvailableModules {
  available_modules: num_bigint::BigUint,
}

impl AvailableModules {
  pub fn union(&self, other: &Self) -> Self {
    Self {
      available_modules: &self.available_modules | &other.available_modules,
    }
  }

  pub fn intersect(&self, other: &Self) -> Self {
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

  pub fn is_empty(&self) -> bool {
    self.available_modules.bits() == 0
  }
}
