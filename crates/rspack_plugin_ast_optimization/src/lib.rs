#![deny(clippy::all)]

use std::path::Path;

use async_trait::async_trait;

use rspack_core::{ast, BundleContext, Plugin, PluginTransformAstHookOutput};
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
  fn need_transform(&self) -> bool {
    false
  }

  #[inline]
  fn need_tap_generated_chunk(&self) -> bool {
    false
  }
  fn transform_ast(
    &self,
    ctx: &BundleContext,
    _path: &Path,
    ast: ast::Module,
  ) -> PluginTransformAstHookOutput {
    Ok(
      ctx
        .compiler
        .run(|| ast.fold_with(&mut constant_folder(ctx.unresolved_mark))),
    )
  }
}
