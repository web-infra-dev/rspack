#![deny(clippy::all)]

use std::path::Path;

use anyhow::Result;
use rspack_core::{ast, BundleContext, Plugin, PluginTransformAstHookOutput};
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

  fn transform_ast(
    &self,
    ctx: &BundleContext,
    _path: &Path,
    ast: ast::Module,
  ) -> PluginTransformAstHookOutput {
    let defintions = &ctx.options.define;
    let mut prefix = DefinePrefix::new(defintions);
    ast.visit_with(&mut prefix);
    let mut define_transform = DefineTransform::new(defintions, prefix);
    Ok(ast.fold_with(&mut define_transform))
  }
}
