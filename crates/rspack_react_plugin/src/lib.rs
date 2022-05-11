use std::path::Path;

use rspack_core::{ast, Bundle, BundleMode, Plugin};
use swc_common::{chain, comments::SingleThreadedComments, Globals, Mark, GLOBALS};
use swc_ecma_transforms_base::resolver::resolver_with_mark;
use swc_ecma_transforms_react as swc_react;
use swc_ecma_visit::FoldWith;
use swc_react::RefreshOptions;

#[derive(Debug)]
pub struct ReactPlugin {
  pub runtime: swc_react::Runtime,
}

pub static PLUGIN_NAME: &'static str = "rspack_react_plugin";

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
      let globals = Globals::new();

      GLOBALS.set(&globals, || {
        let top_level_mark = Mark::fresh(Mark::root());
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
          Mark::from_u32(1),
        );
        let mut folds = chain!(resolver_with_mark(top_level_mark), &mut react_folder);
        ast.fold_with(&mut folds)
      })
    } else {
      ast
    }
  }
}
