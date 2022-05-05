use smol_str::SmolStr;
mod bundle;
mod bundle_context;
mod chunk;
mod dependency_scanner;
pub mod hmr;
mod js_module;
mod module_graph;
mod plugin;
mod plugin_driver;
mod task;
mod utils;
pub use bundle::*;
pub use bundle_context::*;
pub use chunk::*;
pub use js_module::*;
pub use module_graph::*;
pub use plugin::*;
pub use plugin_driver::*;
pub use utils::*;

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
