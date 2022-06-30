#![feature(iter_intersperse)]

mod module;
use std::{ffi::OsStr, path::Path, sync::Arc};

use dashmap::DashSet;
pub use module::*;
mod plugin;
pub use plugin::*;
mod normal_module_factory;
pub use normal_module_factory::*;
mod compiler;
pub use compiler::*;
mod options;
pub use options::*;
mod module_graph;
pub use module_graph::*;
mod chunk;
pub use chunk::*;
mod utils;
pub use utils::*;
mod chunk_graph;
pub use chunk_graph::*;
mod chunk_spliter;
pub use chunk_spliter::*;
mod stats;
pub use stats::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleType {
  Json,
  Css,
  Js,
  Jsx,
  Tsx,
  Ts,
  Unknown,
}

impl ModuleType {
  pub fn is_css(&self) -> bool {
    matches!(self, Self::Css)
  }
}

impl From<&str> for ModuleType {
  fn from(value: &str) -> ModuleType {
    match value {
      "json" => Self::Json,
      "css" => Self::Css,
      "js" => Self::Js,
      "jsx" => Self::Jsx,
      "tsx" => Self::Tsx,
      "ts" => Self::Ts,
      _ => Self::Unknown,
    }
  }
}

pub(crate) type VisitedModuleIdentity = Arc<DashSet<(String, ModuleDependency)>>;
