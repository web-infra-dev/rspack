use std::sync::Arc;

use rspack_core::{
  BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin, Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::loaders::client_entry_loader::{CLIENT_ENTRY_LOADER_IDENTIFIER, ClientEntryLoader};

#[plugin]
#[derive(Debug)]
pub struct ClientEntryLoaderPlugin;

impl ClientEntryLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

impl Default for ClientEntryLoaderPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl Plugin for ClientEntryLoaderPlugin {
  fn name(&self) -> &'static str {
    "ClientEntryLoaderPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));
    Ok(())
  }
}

#[plugin_hook(NormalModuleFactoryResolveLoader for ClientEntryLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;
  if loader_request.starts_with(CLIENT_ENTRY_LOADER_IDENTIFIER) {
    let loader = Arc::new(ClientEntryLoader::new().with_identifier(loader_request.to_string()));
    return Ok(Some(loader));
  }
  Ok(None)
}
