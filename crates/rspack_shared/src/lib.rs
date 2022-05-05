use smol_str::SmolStr;
mod bundle;
mod bundle_context;
mod chunk;
pub mod hmr;
mod js_module;
mod module_graph;
pub use bundle_context::*;
pub use chunk::*;
pub use js_module::*;
pub use module_graph::*;

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
