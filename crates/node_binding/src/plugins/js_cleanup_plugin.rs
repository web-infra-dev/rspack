use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use derive_more::Debug;
use napi::{bindgen_prelude::External, threadsafe_function::ThreadsafeFunction, Status};
use rspack_collections::IdentifierSet;
use rspack_core::{
  ApplyContext, Compilation, CompilationRevokedModules, CompilerFinishMake, CompilerMake,
  CompilerOptions, ModuleIdentifier, PluginContext,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};

pub type CleanupRevokedModulesTsFn = ThreadsafeFunction<
  External<Vec<ModuleIdentifier>>,
  (),
  External<Vec<ModuleIdentifier>>,
  Status,
  false,
  true,
  1,
>;

#[plugin]
#[derive(Debug)]
pub struct JsCleanupPlugin {
  #[debug(skip)]
  cleanup_revoked_modules: CleanupRevokedModulesTsFn,
  in_make_stage: AtomicBool,
}

impl JsCleanupPlugin {
  pub fn new(cleanup_revoked_modules: CleanupRevokedModulesTsFn) -> Self {
    Self::new_inner(cleanup_revoked_modules, AtomicBool::new(false))
  }
}

#[async_trait]
impl rspack_core::Plugin for JsCleanupPlugin {
  fn name(&self) -> &'static str {
    "rspack.JsCleanupPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    ctx
      .context
      .compilation_hooks
      .revoked_modules
      .tap(revoked_modules::new(self));

    ctx.context.compiler_hooks.make.tap(make::new(self));

    ctx
      .context
      .compiler_hooks
      .finish_make
      .tap(finish_make::new(self));

    Ok(())
  }
}

#[plugin_hook(CompilerMake for JsCleanupPlugin)]
async fn make(&self, _compilation: &mut Compilation) -> Result<()> {
  self.in_make_stage.store(true, Ordering::Relaxed);
  Ok(())
}

#[plugin_hook(CompilationRevokedModules for JsCleanupPlugin)]
async fn revoked_modules(&self, revoked_modules: &IdentifierSet) -> rspack_error::Result<()> {
  if !self.in_make_stage.load(Ordering::Relaxed) {
    return Ok(());
  }
  self
    .cleanup_revoked_modules
    .call_async(External::new(
      revoked_modules.iter().cloned().collect::<Vec<_>>(),
    ))
    .await
    .to_rspack_result()?;
  Ok(())
}

#[plugin_hook(CompilerFinishMake for JsCleanupPlugin)]
async fn finish_make(&self, _compilation: &mut Compilation) -> Result<()> {
  self.in_make_stage.store(false, Ordering::Relaxed);
  Ok(())
}
