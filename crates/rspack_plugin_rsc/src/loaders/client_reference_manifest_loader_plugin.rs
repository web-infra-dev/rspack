use std::sync::Arc;

use rspack_core::{
  BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin, Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::loaders::client_reference_manifest_loader::{
  CLIENT_REFERENCE_MANIFEST_LOADER_IDENTIFIER, ClientReferenceManifestLoader,
};

#[plugin]
#[derive(Debug)]
pub struct ClientReferenceManifestLoaderPlugin;

impl ClientReferenceManifestLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

impl Default for ClientReferenceManifestLoaderPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl Plugin for ClientReferenceManifestLoaderPlugin {
  fn name(&self) -> &'static str {
    "ClientReferenceManifestLoaderPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));
    Ok(())
  }
}

#[plugin_hook(NormalModuleFactoryResolveLoader for ClientReferenceManifestLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;
  if loader_request.starts_with(CLIENT_REFERENCE_MANIFEST_LOADER_IDENTIFIER) {
    let loader =
      Arc::new(ClientReferenceManifestLoader::new().with_identifier(loader_request.to_string()));
    return Ok(Some(loader));
  }
  Ok(None)
}
