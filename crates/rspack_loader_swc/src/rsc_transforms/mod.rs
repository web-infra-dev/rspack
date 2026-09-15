mod cjs_finder;
mod import_analyzer;
mod react_server_components;
mod server_actions;
mod to_client_ref;
mod to_server_entry;

#[cfg(test)]
mod tests;

use std::{cell::RefCell, rc::Rc, sync::Arc};

pub use react_server_components::{Config, Options, server_components};
use rspack_core::RscMeta;
pub use server_actions::{Config as ServerActionsConfig, server_actions};
use swc_core::{
  common::{FileName, comments::SingleThreadedComments},
  ecma::ast::Pass,
};
pub use to_server_entry::to_server_entry;

#[derive(Debug, Clone)]
pub(crate) struct RscTransformOptions {
  pub is_react_server_layer: bool,
  pub enable_server_entry: bool,
  pub disable_client_api_checks: bool,
  pub is_development: bool,
  pub hash_salt: String,
}

pub fn rsc_transform(
  filename: Arc<FileName>,
  resource_path: String,
  module_resource: String,
  comments: Rc<SingleThreadedComments>,
  rsc_meta: &RefCell<Option<RscMeta>>,
  options: RscTransformOptions,
) -> impl Pass {
  (
    server_components(
      filename,
      module_resource,
      Config::WithOptions(Options {
        is_react_server_layer: options.is_react_server_layer,
        enable_server_entry: options.enable_server_entry,
        disable_client_api_checks: options.disable_client_api_checks,
      }),
      rsc_meta,
    ),
    server_actions(
      resource_path,
      ServerActionsConfig {
        is_react_server_layer: options.is_react_server_layer,
        is_development: options.is_development,
        hash_salt: options.hash_salt,
      },
      comments,
      rsc_meta,
    ),
  )
}
