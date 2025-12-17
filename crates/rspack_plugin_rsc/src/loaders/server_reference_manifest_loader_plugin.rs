use std::sync::Arc;

use rspack_core::{
  BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin, Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::loaders::server_reference_manifest_loader::{
  SERVER_REFERENCE_MANIFEST_LOADER_IDENTIFIER, ServerReferenceManifestLoader,
};

#[plugin]
#[derive(Debug)]
pub struct ServerReferenceManifestLoaderPlugin;

impl ServerReferenceManifestLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

impl Default for ServerReferenceManifestLoaderPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl Plugin for ServerReferenceManifestLoaderPlugin {
  fn name(&self) -> &'static str {
    "ServerReferenceManifestLoaderPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));
    Ok(())
  }
}

#[plugin_hook(NormalModuleFactoryResolveLoader for ServerReferenceManifestLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;
  if loader_request.starts_with(SERVER_REFERENCE_MANIFEST_LOADER_IDENTIFIER) {
    let loader =
      Arc::new(ServerReferenceManifestLoader::new().with_identifier(loader_request.to_string()));
    return Ok(Some(loader));
  }
  Ok(None)
}
