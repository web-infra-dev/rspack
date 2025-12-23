mod cjs_finder;
mod import_analyzer;
mod react_server_components;
mod server_actions;

use std::{cell::RefCell, sync::Arc};

pub use react_server_components::{Config, Options, server_components};
use rspack_core::{LoaderContext, Module, RscMeta, RunnerContext};
pub use server_actions::{Config as ServerActionsConfig, server_actions};
use swc_core::{
  common::{FileName, comments::SingleThreadedComments},
  ecma::ast::{Pass, noop_pass},
};

pub fn rsc_pass<'a>(
  loader_context: &'a mut LoaderContext<RunnerContext>,
  filename: Arc<FileName>,
  resource_path: &str,
  comments: Arc<SingleThreadedComments>,
  rsc_meta: &RefCell<Option<RscMeta>>,
) -> impl Pass {
  let module = &loader_context.context.module;
  let is_react_server_layer = module
    .get_layer()
    .is_some_and(|layer| layer == "react-server-components");

  (
    // Avoid transforming the redirected server entry module to prevent duplicate RSC metadata generation.
    if loader_context
      .resource_query()
      .is_some_and(|q| q.contains("skip-rsc-transform"))
    {
      swc_core::common::pass::Either::Right(noop_pass())
    } else {
      swc_core::common::pass::Either::Left(server_components(
        filename,
        Config::WithOptions(Options {
          is_react_server_layer,
        }),
        &rsc_meta,
      ))
    },
    server_actions(
      resource_path.to_string(),
      ServerActionsConfig {
        is_react_server_layer,
        is_development: false,
        hash_salt: "".to_string(),
      },
      comments,
      &rsc_meta,
    ),
  )
}
