#![feature(let_chains)]
#![feature(iter_intersperse)]
#![feature(box_patterns)]
#![feature(anonymous_lifetime_in_impl_trait)]

use std::{fmt, sync::Arc};

use rspack_database::Database;
pub mod external_module;
pub use external_module::*;
pub mod ast;
pub mod cache;
mod missing_module;
pub use missing_module::*;
mod normal_module;
mod raw_module;
pub use raw_module::*;
pub mod module;
pub mod parser_and_generator;
pub use module::*;
pub use parser_and_generator::*;
mod runtime_globals;
pub use normal_module::*;
pub use runtime_globals::RuntimeGlobals;
mod plugin;
pub use plugin::*;
mod context_module;
pub use context_module::*;
mod context_module_factory;
pub use context_module_factory::*;
mod init_fragment;
pub use init_fragment::*;
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
mod build_chunk_graph;
mod stats;
pub use stats::*;
mod runtime;
mod runtime_module;
pub use runtime::*;
pub use runtime_module::*;
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
mod module_graph_module;
pub use module_graph_module::*;
pub mod tree_shaking;

pub use rspack_loader_runner::{get_scheme, ResourceData, Scheme};
pub use rspack_sources;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceType {
  JavaScript,
  Css,
  Wasm,
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
  WasmSync,
  WasmAsync,
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
      ModuleType::Js | ModuleType::JsEsm | ModuleType::JsDynamic | ModuleType::Ts
    ) || self.is_jsx_like()
  }

  pub fn is_asset_like(&self) -> bool {
    matches!(
      self,
      ModuleType::Asset | ModuleType::AssetInline | ModuleType::AssetResource
    )
  }
  pub fn is_jsx_like(&self) -> bool {
    matches!(
      self,
      ModuleType::Tsx | ModuleType::Jsx | ModuleType::JsxEsm | ModuleType::JsxDynamic
    )
  }

  pub fn is_wasm_like(&self) -> bool {
    matches!(self, ModuleType::WasmSync | ModuleType::WasmAsync)
  }

  pub fn is_js_auto(&self) -> bool {
    matches!(
      self,
      ModuleType::Js | ModuleType::Jsx | ModuleType::Ts | ModuleType::Tsx
    )
  }

  pub fn is_js_esm(&self) -> bool {
    matches!(
      self,
      ModuleType::JsEsm | ModuleType::JsxEsm | ModuleType::Ts | ModuleType::Tsx
    )
  }

  pub fn is_js_dynamic(&self) -> bool {
    matches!(
      self,
      ModuleType::JsDynamic | ModuleType::JsxDynamic | ModuleType::Ts | ModuleType::Tsx
    )
  }
}

impl fmt::Display for ModuleType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        ModuleType::Js => "javascript/auto",
        ModuleType::JsEsm => "javascript/esm",
        ModuleType::JsDynamic => "javascript/dynamic",

        ModuleType::Jsx => "javascriptx",
        ModuleType::JsxEsm => "javascriptx/esm",
        ModuleType::JsxDynamic => "javascriptx/dynamic",

        ModuleType::Ts => "typescript",
        ModuleType::Tsx => "typescriptx",

        ModuleType::Css => "css",
        ModuleType::CssModule => "css/module",

        ModuleType::Json => "json",

        ModuleType::WasmSync => "webassembly/sync",
        ModuleType::WasmAsync => "webassembly/async",

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
      "mjs" => Ok(Self::JsEsm),
      "cjs" => Ok(Self::JsDynamic),
      "js" | "javascript" | "js/auto" | "javascript/auto" => Ok(Self::Js),
      "js/dynamic" | "javascript/dynamic" => Ok(Self::JsDynamic),
      "js/esm" | "javascript/esm" => Ok(Self::JsEsm),

      "jsx" | "javascriptx" | "jsx/auto" | "javascriptx/auto" => Ok(Self::Jsx),
      "jsx/dynamic" | "javascriptx/dynamic" => Ok(Self::JsxDynamic),
      "jsx/esm" | "javascriptx/esm" => Ok(Self::JsxEsm),

      "ts" | "typescript" => Ok(Self::Ts),
      "tsx" | "typescriptx" => Ok(Self::Tsx),

      "css" => Ok(Self::Css),
      "css/module" => Ok(Self::CssModule),

      "json" => Ok(Self::Json),

      "webassembly/sync" => Ok(Self::WasmSync),
      "webassembly/async" => Ok(Self::WasmAsync),

      "asset" => Ok(Self::Asset),
      "asset/resource" => Ok(Self::AssetResource),
      "asset/source" => Ok(Self::AssetSource),
      "asset/inline" => Ok(Self::AssetInline),

      _ => {
        use rspack_error::internal_error;
        Err(internal_error!("invalid module type: {value}"))
      }
    }
  }
}

pub type ChunkByUkey = Database<Chunk>;
pub type ChunkGroupByUkey = Database<ChunkGroup>;
pub(crate) type SharedPluginDriver = Arc<RwLock<PluginDriver>>;
