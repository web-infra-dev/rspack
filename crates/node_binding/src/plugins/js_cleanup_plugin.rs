use async_trait::async_trait;
use derive_more::Debug;
use napi::bindgen_prelude::External;
use napi::threadsafe_function::ThreadsafeFunction;
use rspack_collections::IdentifierSet;
use rspack_core::{ApplyContext, CompilerOptions, PluginContext};
use rspack_core::{CompilationRevokedModules, ModuleIdentifier};
use rspack_hook::plugin;
use rspack_hook::plugin_hook;
use rspack_napi::NapiResultExt;

pub type CleanupRevokedModulesTsFn = ThreadsafeFunction<
  External<Vec<ModuleIdentifier>>,
  (),
  External<Vec<ModuleIdentifier>>,
  false,
  true,
  1,
>;

#[plugin]
#[derive(Debug)]
pub struct JsCleanupPlugin {
  #[debug(skip)]
  cleanup_revoked_modules: CleanupRevokedModulesTsFn,
}

impl JsCleanupPlugin {
  pub fn new(cleanup_revoked_modules: CleanupRevokedModulesTsFn) -> Self {
    Self::new_inner(cleanup_revoked_modules)
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

    Ok(())
  }
}

#[plugin_hook(CompilationRevokedModules for JsCleanupPlugin)]
async fn revoked_modules(&self, revoked_modules: &IdentifierSet) -> rspack_error::Result<()> {
  self
    .cleanup_revoked_modules
    .call_async(External::new(
      revoked_modules.iter().cloned().collect::<Vec<_>>(),
    ))
    .await
    .into_rspack_result()?;
  Ok(())
}
