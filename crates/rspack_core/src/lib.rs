#![feature(let_chains)]
#![feature(iter_intersperse)]
#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(anonymous_lifetime_in_impl_trait)]

use std::{fmt, sync::Arc};

use rspack_database::Database;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

pub mod ast;
pub mod cache;
mod missing_module;
pub use missing_module::*;
mod normal_module;
mod raw_module;
pub use raw_module::*;
pub mod identifier;
pub mod module;
pub use identifier::*;
pub mod parser_and_generator;
pub use module::*;
pub use parser_and_generator::*;
pub mod runtime_globals;
pub use normal_module::*;
mod plugin;
pub use plugin::*;
mod context_module;
pub use context_module::*;
mod context_module_factory;
pub use context_module_factory::*;

mod module_factory;
pub use module_factory::*;
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
mod dependency;
pub use dependency::*;
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
mod code_generation_results;
pub use code_generation_results::*;
mod entrypoint;
pub use entrypoint::*;
mod loader;
pub use loader::*;
// mod external;
// pub use external::*;
mod chunk_group;
pub use chunk_group::*;
mod ukey;
pub use ukey::*;

pub mod tree_shaking;

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
  CssModule,
  Js,
  JsDynamic,
  JsEsm,
  Jsx,
  JsxDynamic,
  JsxEsm,
  Tsx,
  Ts,
  AssetInline,
  AssetResource,
  AssetSource,
  Asset,
}

impl ModuleType {
  pub fn is_css_like(&self) -> bool {
    matches!(self, Self::Css | Self::CssModule)
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

impl fmt::Display for ModuleType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        ModuleType::Js => "js",
        ModuleType::JsEsm => "js/esm",
        ModuleType::JsDynamic => "js/dynamic",

        ModuleType::Jsx => "jsx",
        ModuleType::JsxEsm => "jsx/esm",
        ModuleType::JsxDynamic => "jsx/dynamic",

        ModuleType::Ts => "ts",
        ModuleType::Tsx => "tsx",

        ModuleType::Css => "css",
        ModuleType::CssModule => "css/module",

        ModuleType::Json => "json",

        ModuleType::Asset => "asset",
        ModuleType::AssetSource => "asset/source",
        ModuleType::AssetResource => "asset/resource",
        ModuleType::AssetInline => "asset/inline",
      }
    )
  }
}

impl TryFrom<&str> for ModuleType {
  type Error = rspack_error::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      // TODO: change to esm
      "mjs" => Ok(Self::Js),
      "js" | "javascript" | "js/auto" | "javascript/auto" => Ok(Self::Js),
      "js/dynamic" | "javascript/dynamic" => Ok(Self::JsDynamic),
      "js/esm" | "javascript/esm" => Ok(Self::JsEsm),

      "jsx" | "javascriptx" | "jsx/auto" | "javascriptx/auto" => Ok(Self::Jsx),
      "jsx/dynamic" | "javascriptx/dynamic" => Ok(Self::JsxDynamic),
      "jsx/esm" | "javascriptx/esm" => Ok(Self::JsxEsm),

      "ts" => Ok(Self::Ts),
      "tsx" => Ok(Self::Tsx),

      "css" => Ok(Self::Css),
      "css/module" => Ok(Self::CssModule),

      "json" => Ok(Self::Json),

      "asset" => Ok(Self::Asset),
      "asset/resource" => Ok(Self::AssetResource),
      "asset/source" => Ok(Self::AssetSource),
      "asset/inline" => Ok(Self::AssetInline),

      // default
      "png" => Ok(Self::Asset),
      "jpeg" => Ok(Self::Asset),
      "jpg" => Ok(Self::Asset),
      "svg" => Ok(Self::Asset),
      "txt" => Ok(Self::AssetSource),
      // TODO: get module_type from module
      "scss" | "sass" => Ok(Self::Css),
      _ => {
        use rspack_error::internal_error;
        Err(rspack_error::Error::InternalError(internal_error!(
          format!("invalid module type: {value}")
        )))
      }
    }
  }
}

// TODO: use module identifier only later, (ModuleIdentifier, DependencyCategory, Specifier)
pub(crate) type VisitedModuleIdentity = HashSet<(ModuleIdentifier, DependencyCategory, String)>;

pub type ChunkByUkey = Database<Chunk>;
pub type ChunkGroupByUkey = HashMap<ChunkGroupUkey, ChunkGroup>;
pub(crate) type SharedPluginDriver = Arc<RwLock<PluginDriver>>;
