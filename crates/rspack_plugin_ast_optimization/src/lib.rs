#![deny(clippy::all)]

use std::path::Path;

use async_trait::async_trait;

use rspack_core::{ast, BundleContext, BundleMode, Plugin, PluginTransformAstHookOutput};
use rspack_swc::swc_ecma_visit::FoldWith;

mod constant_folder;
mod utils;

use constant_folder::constant_folder;

// TODO:
// struct OptimizationOptions;

#[derive(Debug)]
pub struct OptimizationPlugin;

impl OptimizationPlugin {
  pub fn new() -> Self {
    Self
  }
}

impl Default for OptimizationPlugin {
  fn default() -> Self {
    Self::new()
  }
}

#[async_trait]
impl Plugin for OptimizationPlugin {
  fn name(&self) -> &'static str {
    "rspack_plugin_ast_optimization"
  }

  fn transform_ast(
    &self,
    ctx: &BundleContext,
    _path: &Path,
    ast: ast::Module,
  ) -> PluginTransformAstHookOutput {
    ctx
      .compiler
      .run(|| ast.fold_with(&mut constant_folder(ctx.unresolved_mark)))
  }
}
