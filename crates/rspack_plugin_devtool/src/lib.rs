#![feature(let_chains)]

mod eval_dev_tool_module_plugin;
mod eval_source_map_dev_tool_plugin;
mod mapped_assets_cache;
mod module_filename_helpers;
mod source_map_dev_tool_module_options_plugin;
mod source_map_dev_tool_plugin;

use std::sync::Arc;

pub use eval_dev_tool_module_plugin::*;
pub use eval_source_map_dev_tool_plugin::*;
use futures::future::BoxFuture;
use rspack_core::ModuleIdentifier;
use rspack_error::Result;
pub use source_map_dev_tool_module_options_plugin::*;
pub use source_map_dev_tool_plugin::*;

pub type ModuleFilenameTemplateFn =
  Arc<dyn Fn(ModuleFilenameTemplateFnCtx) -> BoxFuture<'static, Result<String>> + Sync + Send>;

pub struct ModuleFilenameTemplateFnCtx {
  pub identifier: String,
  pub short_identifier: String,
  pub resource: String,
  pub resource_path: String,
  pub absolute_resource_path: String,
  pub loaders: String,
  pub all_loaders: String,
  pub query: String,
  pub module_id: String,
  pub hash: String,
  pub namespace: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum ModuleOrSource {
  Source(String),
  Module(ModuleIdentifier),
}
