use smol_str::SmolStr;
mod bundle_context;
mod chunk;
pub mod hmr;
mod js_module;
pub use bundle_context::*;
pub use chunk::*;
pub use js_module::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ResolvedId {
  pub id: SmolStr,
  pub external: bool,
}

impl ResolvedId {
  pub fn new<T: Into<SmolStr>>(id: T, external: bool) -> Self {
    Self {
      id: id.into(),
      external,
      // module_side_effects: false,
    }
  }
}
