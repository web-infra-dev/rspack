#![feature(iter_intersperse)]

mod module;
use std::sync::Arc;

use dashmap::DashSet;
pub use module::*;
mod plugin;
pub use plugin::*;
mod resolving_module_job;
pub use resolving_module_job::*;
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
pub enum SourceType {
  Json,
  Css,
  Js,
  Jsx,
  Tsx,
  Ts,
}

impl SourceType {
  pub fn is_css(&self) -> bool {
    matches!(self, Self::Css)
  }
}

impl TryFrom<&str> for SourceType {
  type Error = ();

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "json" => Ok(Self::Json),
      "css" => Ok(Self::Css),
      "js" => Ok(Self::Js),
      "jsx" => Ok(Self::Jsx),
      "tsx" => Ok(Self::Tsx),
      "ts" => Ok(Self::Ts),
      _ => Err(()),
    }
  }
}

pub(crate) type VisitedModuleIdentity = Arc<DashSet<(String, ModuleDependency)>>;
