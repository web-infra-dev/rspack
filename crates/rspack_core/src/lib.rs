#![feature(box_patterns)]

mod bundle;
mod bundle_context;
mod chunk;
mod dependency_scanner;
pub mod hmr;
mod js_module;
mod module_graph;
mod options;
mod plugin;
mod plugin_driver;
mod task;
mod utils;
pub use bundle::*;
pub use bundle_context::*;
pub use chunk::*;
pub use js_module::*;
pub use module_graph::*;
use once_cell::sync::Lazy;
pub use options::*;
pub use plugin::*;
pub use plugin_driver::*;
use rspack_swc::swc_common;
pub use rspack_swc::swc_ecma_ast as ast;
use swc_common::Globals;
pub use utils::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ResolvedURI {
  pub uri: String,
  pub external: bool,
}

impl ResolvedURI {
  pub fn new<T: Into<String>>(path: T, external: bool) -> Self {
    Self {
      uri: path.into(),
      external,
      // module_side_effects: false,
    }
  }
}
