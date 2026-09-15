use std::sync::Arc;

use derive_more::Debug;
use futures::future::BoxFuture;
use itertools::Itertools;
use rspack_core::{
  CircularModulesInfo, Compilation, CompilationOptimizeModules, CompilerId, CompilerMake, Module,
  ModuleIdentifier, Plugin,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_regex::RspackRegex;

pub type CircularCheckHandlerFn = Arc<
  dyn for<'a> Fn(CompilerId, &'a dyn Module, Vec<String>) -> BoxFuture<'a, Result<()>>
    + Send
    + Sync,
>;

#[derive(Debug, Default)]
pub struct CircularCheckRspackPluginOptions {
  pub exclude: Option<RspackRegex>,
  pub include: Option<RspackRegex>,
  pub fail_on_error: bool,
  #[debug(skip)]
  pub on_detected: Option<CircularCheckHandlerFn>,
}

#[plugin]
#[derive(Debug)]
pub struct CircularCheckRspackPlugin {
  options: CircularCheckRspackPluginOptions,
}

impl CircularCheckRspackPlugin {
  pub fn new(options: CircularCheckRspackPluginOptions) -> Self {
    Self::new_inner(options)
  }

  fn readable_cycle_paths(
    &self,
    compilation: &Compilation,
    path: &[ModuleIdentifier],
  ) -> Option<Vec<String>> {
    if path.is_empty() {
      return None;
    }

    Some(
      path
        .iter()
        .filter_map(|module_id| {
          compilation.module_by_identifier(module_id).map(|module| {
            module
              .readable_identifier(&compilation.options.context)
              .into_owned()
          })
        })
        .collect(),
    )
  }

  fn is_ignored(&self, paths: &[String]) -> bool {
    if self
      .options
      .exclude
      .as_ref()
      .is_some_and(|exclude| paths.iter().any(|path| exclude.test(path)))
    {
      return true;
    }

    self
      .options
      .include
      .as_ref()
      .is_some_and(|include| !paths.iter().any(|path| include.test(path)))
  }

  async fn handle_detected(
    &self,
    compilation: &Compilation,
    module_id: &ModuleIdentifier,
    paths: Vec<String>,
    diagnostics: &mut Vec<Diagnostic>,
  ) -> Result<()> {
    let Some(module) = compilation.module_by_identifier(module_id) else {
      return Ok(());
    };

    if let Some(callback) = &self.options.on_detected {
      return callback(compilation.compiler_id(), module.as_ref(), paths).await;
    }

    let diagnostic_factory = if self.options.fail_on_error {
      Diagnostic::error
    } else {
      Diagnostic::warn
    };

    diagnostics.push(diagnostic_factory(
      "Circular Dependency".to_string(),
      format!(
        "Circular dependency detected:\n {}",
        paths.iter().join(" -> ")
      ),
    ));
    Ok(())
  }
}

#[plugin_hook(CompilerMake for CircularCheckRspackPlugin)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  compilation.circular_modules.enable_collect_cycle_paths();
  Ok(())
}

#[plugin_hook(CompilationOptimizeModules for CircularCheckRspackPlugin)]
async fn optimize_modules(
  &self,
  compilation: &Compilation,
  circular_modules: &mut CircularModulesInfo,
  diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<bool>> {
  circular_modules.ensure_circular_modules_info(compilation);

  let cycle_paths = circular_modules
    .cycle_paths()
    .expect("should have cycle_paths");

  for path in cycle_paths {
    let Some(module_id) = path.first() else {
      continue;
    };
    let Some(paths) = self.readable_cycle_paths(compilation, path) else {
      continue;
    };

    if self.is_ignored(&paths) {
      continue;
    }

    self
      .handle_detected(compilation, module_id, paths, diagnostics)
      .await?;
  }

  Ok(None)
}

impl Plugin for CircularCheckRspackPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.make.tap(make::new(self));
    ctx
      .compilation_hooks
      .optimize_modules
      .tap(optimize_modules::new(self));
    Ok(())
  }
}
