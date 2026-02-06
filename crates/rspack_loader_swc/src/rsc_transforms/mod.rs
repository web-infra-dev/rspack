mod cjs_finder;
mod import_analyzer;
mod react_server_components;
mod server_actions;
mod to_module_ref;

use std::{cell::RefCell, rc::Rc, sync::Arc};

pub(crate) use react_server_components::{Config, Options, server_components};
use rspack_core::{LoaderContext, Module, RscMeta, RunnerContext};
pub(crate) use server_actions::{Config as ServerActionsConfig, server_actions};
use swc_core::{
  common::{FileName, comments::SingleThreadedComments},
  ecma::ast::Pass,
};
pub(crate) use to_module_ref::to_module_ref;

pub(crate) fn rsc_pass(
  loader_context: &mut LoaderContext<RunnerContext>,
  filename: Arc<FileName>,
  resource_path: &str,
  comments: Rc<SingleThreadedComments>,
  rsc_meta: &RefCell<Option<RscMeta>>,
) -> impl Pass {
  let module = &loader_context.context.module;
  let is_react_server_layer = module
    .get_layer()
    .is_some_and(|layer| layer == "react-server-components");

  // Avoid transforming the redirected server entry module to prevent duplicate RSC metadata generation.
  let server_entry_proxy = loader_context
    .resource_query()
    .is_some_and(|q| q.contains("rsc-server-entry-proxy=true"));

  (
    server_components(
      filename,
      Config::WithOptions(Options {
        is_react_server_layer,
        enable_server_entry: !server_entry_proxy,
      }),
      rsc_meta,
    ),
    server_actions(
      resource_path.to_string(),
      ServerActionsConfig {
        is_react_server_layer,
        is_development: false,
        hash_salt: String::new(),
      },
      comments,
      rsc_meta,
    ),
  )
}
