#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(iter_intersperse)]
#![feature(box_patterns)]
#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(hash_raw_entry)]
#![feature(option_get_or_insert_default)]
#![feature(slice_group_by)]
use std::{fmt, sync::Arc};
mod dependencies_block;
pub mod diagnostics;
mod update_hash;
pub use dependencies_block::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, DependenciesBlock, DependencyLocation,
};
mod fake_namespace_object;
mod template;
pub use fake_namespace_object::*;
pub use template::Template;
mod module_profile;
pub use module_profile::*;
use rspack_database::Database;
pub mod external_module;
pub use external_module::*;
mod logger;
pub use logger::*;
pub mod cache;
mod normal_module;
mod raw_module;
pub use raw_module::*;
mod exports_info;
pub use exports_info::*;
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
mod ignore_error_module_factory;
pub use ignore_error_module_factory::*;
mod self_module_factory;
pub use self_module_factory::*;
mod self_module;
pub use self_module::*;
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
use ustr::Ustr;
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
pub mod resolver;
pub use resolver::*;
pub mod concatenated_module;
pub mod reserved_names;
pub mod tree_shaking;

pub use rspack_core_macros::{impl_runtime_module, impl_source_map_config};
pub use rspack_loader_runner::{get_scheme, ResourceData, Scheme, BUILTIN_LOADER_PREFIX};
pub use rspack_sources;

#[cfg(debug_assertions)]
pub mod debug_info;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceType {
  JavaScript,
  Css,
  Wasm,
  Asset,
  Expose,
  Remote,
  ShareInit,
  ConsumeShared,
  Custom(Ustr),
  #[default]
  Unknown,
}

impl std::fmt::Display for SourceType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SourceType::JavaScript => write!(f, "javascript"),
      SourceType::Css => write!(f, "css"),
      SourceType::Wasm => write!(f, "wasm"),
      SourceType::Asset => write!(f, "asset"),
      SourceType::Expose => write!(f, "expose"),
      SourceType::Remote => write!(f, "remote"),
      SourceType::ShareInit => write!(f, "share-init"),
      SourceType::ConsumeShared => write!(f, "consume-shared"),
      SourceType::Unknown => write!(f, "unknown"),
      SourceType::Custom(source_type) => f.write_str(source_type),
    }
  }
}

impl From<&str> for SourceType {
  fn from(value: &str) -> Self {
    match value {
      "javascript" => Self::JavaScript,
      "css" => Self::Css,
      "wasm" => Self::Wasm,
      "asset" => Self::Asset,
      "expose" => Self::Expose,
      "remote" => Self::Remote,
      "share-init" => Self::ShareInit,
      "consume-shared" => Self::ConsumeShared,
      "unknown" => Self::Unknown,
      other => SourceType::Custom(other.into()),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleType {
  Json,
  Css,
  CssModule,
  CssAuto,
  Js,
  JsDynamic,
  JsEsm,
  WasmSync,
  WasmAsync,
  AssetInline,
  AssetResource,
  AssetSource,
  Asset,
  Runtime,
  Remote,
  Fallback,
  ProvideShared,
  ConsumeShared,
  SelfReference,
  Custom(Ustr),
}

impl ModuleType {
  pub fn is_css_like(&self) -> bool {
    matches!(self, Self::Css | Self::CssModule | Self::CssAuto)
  }

  pub fn is_js_like(&self) -> bool {
    matches!(
      self,
      ModuleType::Js | ModuleType::JsEsm | ModuleType::JsDynamic
    )
  }

  pub fn is_asset_like(&self) -> bool {
    matches!(
      self,
      ModuleType::Asset | ModuleType::AssetInline | ModuleType::AssetResource
    )
  }

  pub fn is_wasm_like(&self) -> bool {
    matches!(self, ModuleType::WasmSync | ModuleType::WasmAsync)
  }

  pub fn is_js_auto(&self) -> bool {
    matches!(self, ModuleType::Js)
  }

  pub fn is_js_esm(&self) -> bool {
    matches!(self, ModuleType::JsEsm)
  }

  pub fn is_js_dynamic(&self) -> bool {
    matches!(self, ModuleType::JsDynamic)
  }

  /// Webpack arbitrary determines the binary type from [NormalModule.binary](https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/NormalModule.js#L302)
  pub fn is_binary(&self) -> bool {
    self.is_asset_like() || self.is_wasm_like()
  }

  pub fn as_str(&self) -> &str {
    match self {
      ModuleType::Js => "javascript/auto",
      ModuleType::JsEsm => "javascript/esm",
      ModuleType::JsDynamic => "javascript/dynamic",

      ModuleType::Css => "css",
      ModuleType::CssModule => "css/module",
      ModuleType::CssAuto => "css/auto",

      ModuleType::Json => "json",

      ModuleType::WasmSync => "webassembly/sync",
      ModuleType::WasmAsync => "webassembly/async",

      ModuleType::Asset => "asset",
      ModuleType::AssetSource => "asset/source",
      ModuleType::AssetResource => "asset/resource",
      ModuleType::AssetInline => "asset/inline",
      ModuleType::Runtime => "runtime",
      ModuleType::Remote => "remote-module",
      ModuleType::Fallback => "fallback-module",
      ModuleType::ProvideShared => "provide-module",
      ModuleType::ConsumeShared => "consume-shared-module",
      ModuleType::SelfReference => "self-reference-module",

      ModuleType::Custom(custom) => custom,
    }
  }
}

impl fmt::Display for ModuleType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str(),)
  }
}

impl From<&str> for ModuleType {
  fn from(value: &str) -> Self {
    match value {
      "mjs" => Self::JsEsm,
      "cjs" => Self::JsDynamic,
      "js" | "javascript" | "js/auto" | "javascript/auto" => Self::Js,
      "js/dynamic" | "javascript/dynamic" => Self::JsDynamic,
      "js/esm" | "javascript/esm" => Self::JsEsm,

      "css" => Self::Css,
      "css/module" => Self::CssModule,
      "css/auto" => Self::CssAuto,

      "json" => Self::Json,

      "webassembly/sync" => Self::WasmSync,
      "webassembly/async" => Self::WasmAsync,

      "asset" => Self::Asset,
      "asset/resource" => Self::AssetResource,
      "asset/source" => Self::AssetSource,
      "asset/inline" => Self::AssetInline,

      custom => Self::Custom(custom.into()),
    }
  }
}

pub type ChunkByUkey = Database<Chunk>;
pub type ChunkGroupByUkey = Database<ChunkGroup>;
pub(crate) type SharedPluginDriver = Arc<PluginDriver>;

pub fn get_chunk_group_from_ukey<'a>(
  ukey: &ChunkGroupUkey,
  chunk_group_by_ukey: &'a ChunkGroupByUkey,
) -> Option<&'a ChunkGroup> {
  if chunk_group_by_ukey.contains(ukey) {
    Some(chunk_group_by_ukey.expect_get(ukey))
  } else {
    None
  }
}

pub fn get_chunk_from_ukey<'a>(
  ukey: &ChunkUkey,
  chunk_by_ukey: &'a ChunkByUkey,
) -> Option<&'a Chunk> {
  if chunk_by_ukey.contains(ukey) {
    Some(chunk_by_ukey.expect_get(ukey))
  } else {
    None
  }
}

pub fn get_mut_chunk_from_ukey<'a>(
  ukey: &ChunkUkey,
  chunk_by_ukey: &'a mut ChunkByUkey,
) -> Option<&'a mut Chunk> {
  if chunk_by_ukey.contains(ukey) {
    Some(chunk_by_ukey.expect_get_mut(ukey))
  } else {
    None
  }
}
