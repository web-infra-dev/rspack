#![feature(iter_intersperse)]

use std::sync::Arc;

use dashmap::DashSet;
use hashbrown::HashMap;

mod normal_module;
pub use normal_module::*;
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
use tokio::sync::RwLock;
pub use utils::*;
mod chunk_graph;
pub use chunk_graph::*;
mod chunk_spliter;
pub use chunk_spliter::*;
mod stats;
pub use stats::*;
mod runtime;
pub use runtime::*;
mod entrypoint;
pub use entrypoint::*;
mod loader;
pub use loader::*;
mod external;
pub use external::*;
mod chunk_group;
pub use chunk_group::*;
mod ukey;
pub use ukey::*;

pub use rspack_sources;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceType {
  JavaScript,
  Css,
  Asset,
  #[default]
  Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleType {
  Json,
  Css,
  Js,
  Jsx,
  Tsx,
  Ts,
  AssetInline,
  AssetResource,
  AssetSource,
  Asset,
}

impl ModuleType {
  pub fn is_css(&self) -> bool {
    matches!(self, Self::Css)
  }

  pub fn is_js_like(&self) -> bool {
    matches!(
      self,
      ModuleType::Js | ModuleType::Ts | ModuleType::Tsx | ModuleType::Jsx
    )
  }

  pub fn is_asset_like(&self) -> bool {
    matches!(
      self,
      ModuleType::Asset | ModuleType::AssetInline | ModuleType::AssetResource
    )
  }
  pub fn is_jsx_like(&self) -> bool {
    matches!(self, ModuleType::Tsx | ModuleType::Jsx)
  }
}

impl TryFrom<&str> for ModuleType {
  type Error = ();

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "json" => Ok(Self::Json),
      "css" => Ok(Self::Css),
      "mjs" => Ok(Self::Js),
      "js" => Ok(Self::Js),
      "jsx" => Ok(Self::Jsx),
      "tsx" => Ok(Self::Tsx),
      "ts" => Ok(Self::Ts),

      // default
      "png" => Ok(Self::Asset),
      "jpeg" => Ok(Self::Asset),
      "jpg" => Ok(Self::Asset),
      "svg" => Ok(Self::Asset),
      "txt" => Ok(Self::AssetSource),
      // TODO: get module_type from module
      "scss" | "sass" => Ok(Self::Css),
      _ => Err(()),
    }
  }
}

pub(crate) type VisitedModuleIdentity = Arc<DashSet<(String, ModuleDependency)>>;

pub(crate) type ChunkByUkey = HashMap<ChunkUkey, Chunk>;
pub type ChunkGroupByUkey = HashMap<ChunkGroupUkey, ChunkGroup>;
pub(crate) type SharedPluginDriver = Arc<RwLock<PluginDriver>>;
