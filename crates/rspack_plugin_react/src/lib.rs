use std::path::Path;

use rspack_core::{ast, BundleMode, Plugin, SWC_GLOBALS};
use rspack_swc::{swc_common, swc_ecma_transforms_react as swc_react, swc_ecma_visit};
use swc_common::{comments::SingleThreadedComments, GLOBALS};
use swc_ecma_visit::FoldWith;
use swc_react::RefreshOptions;

pub static PLUGIN_NAME: &'static str = "rspack_plugin_react";

#[derive(Debug)]
pub struct ReactPlugin {
  pub runtime: swc_react::Runtime,
}

impl Plugin for ReactPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn transform(
    &self,
    ctx: &rspack_core::BundleContext,
    path: &Path,
    ast: ast::Module,
  ) -> rspack_core::PluginTransformHookOutput {
    let is_maybe_has_jsx = path.extension().map_or(true, |ext| ext != "ts");
    if is_maybe_has_jsx {
      GLOBALS.set(&SWC_GLOBALS, || {
        let mut react_folder = swc_react::react::<SingleThreadedComments>(
          ctx.compiler.cm.clone(),
          None,
          swc_react::Options {
            development: matches!(ctx.options.mode, BundleMode::Dev),
            runtime: Some(self.runtime),
            refresh: if ctx.options.react.refresh {
              Some(RefreshOptions {
                ..Default::default()
              })
            } else {
              None
            },
            ..Default::default()
          },
          ctx.top_level_mark.clone(),
        );
        let mut folds = &mut react_folder;
        ast.fold_with(&mut folds)
      })
    } else {
      ast
    }
  }
}
