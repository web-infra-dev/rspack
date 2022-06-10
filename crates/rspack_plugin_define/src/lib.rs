#![deny(clippy::all)]

use rspack_core::{ast, Plugin, PluginContext, PluginTransformAstHookOutput};
use rspack_swc::swc_ecma_visit::{FoldWith, VisitWith};

mod prefix;
mod transfrom;
use prefix::DefinePrefix;
use transfrom::DefineTransform;

#[derive(Debug)]
pub struct DefinePlugin {}

impl Plugin for DefinePlugin {
  fn name(&self) -> &'static str {
    "rspack_define_plugin"
  }

  #[inline]
  fn need_build_start(&self) -> bool {
    false
  }

  #[inline]
  fn need_build_end(&self) -> bool {
    false
  }

  #[inline]
  fn need_resolve(&self) -> bool {
    false
  }

  #[inline]
  fn need_load(&self) -> bool {
    false
  }

  #[inline]
  fn need_tap_generated_chunk(&self) -> bool {
    false
  }

  fn optimize_ast(
    &self,
    ctx: &PluginContext,
    _path: &str,
    ast: ast::Module,
  ) -> PluginTransformAstHookOutput {
    let defintions = &ctx.options.define;
    let mut prefix = DefinePrefix::new(defintions);
    ast.visit_with(&mut prefix);
    let mut define_transform = DefineTransform::new(defintions, prefix);
    Ok(ast.fold_with(&mut define_transform))
  }
}
