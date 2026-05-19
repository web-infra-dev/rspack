use async_trait::async_trait;
use derive_more::Debug;
use napi::{Status, bindgen_prelude::External, threadsafe_function::ThreadsafeFunction};
use rspack_collections::IdentifierSet;
use rspack_core::{
  ApplyContext, CompilationRevokedModules, CompilerId, CompilerOptions, ModuleIdentifier, Plugin,
};
use rspack_error::ToStringResultToRspackResultExt;
use rspack_hook::{plugin, plugin_hook};

pub type CleanupRevokedModulesTsFn = ThreadsafeFunction<
  External<(CompilerId, Vec<ModuleIdentifier>)>,
  (),
  External<(CompilerId, Vec<ModuleIdentifier>)>,
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
}

impl JsCleanupPlugin {
  pub fn new(cleanup_revoked_modules: CleanupRevokedModulesTsFn) -> Self {
    Self::new_inner(cleanup_revoked_modules)
  }
}

impl Plugin for JsCleanupPlugin {
  fn name(&self) -> &'static str {
    "rspack.JsCleanupPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> rspack_error::Result<()> {
    ctx
      .compilation_hooks
      .revoked_modules
      .tap(revoked_modules::new(self));

    Ok(())
  }
}

#[plugin_hook(CompilationRevokedModules for JsCleanupPlugin)]
async fn revoked_modules(
  &self,
  compilation: &rspack_core::Compilation,
  revoked_modules: &IdentifierSet,
) -> rspack_error::Result<()> {
  if !revoked_modules.is_empty() {
    self
      .cleanup_revoked_modules
      .call_async(External::new((
        compilation.compiler_id(),
        revoked_modules.iter().copied().collect::<Vec<_>>(),
      )))
      .await
      .to_rspack_result()?;
  }
  Ok(())
}
