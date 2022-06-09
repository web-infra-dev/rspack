#![feature(iter_intersperse)]

mod module;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceType {
  Json,
  Css,
  Js,
  Jsx,
  Tsx,
  Ts,
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
