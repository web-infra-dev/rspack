use std::path::Path;

use rspack_core::{ast, Plugin};
use swc_common::{comments::SingleThreadedComments, Globals, Mark, GLOBALS};
use swc_ecma_transforms_react as swc_react;
use swc_ecma_visit::FoldWith;

#[derive(Debug)]
pub struct ReactPlugin {
  pub runtime: swc_react::Runtime,
}

impl Plugin for ReactPlugin {
  fn name(&self) -> &'static str {
    "rspack_plugin_react"
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
        let mut react_folder = swc_react::react::<SingleThreadedComments>(
          ctx.compiler.cm.clone(),
          None,
          swc_react::Options {
            runtime: Some(self.runtime),
            ..Default::default()
          },
          Mark::from_u32(1),
        );
        ast.fold_with(&mut react_folder)
      })
    } else {
      ast
    }
  }
}
