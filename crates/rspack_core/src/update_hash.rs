use std::hash::Hasher;

use crate::{Compilation, RuntimeSpec};

pub struct UpdateHashContext<'a> {
  pub compilation: &'a Compilation,
  pub runtime: Option<&'a RuntimeSpec>,
}

pub trait UpdateRspackHash {
  fn update_hash<H: Hasher>(&self, state: &mut H, context: &UpdateHashContext);
}
