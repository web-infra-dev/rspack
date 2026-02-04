use std::sync::Arc;

use rspack_core::{
  BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin, Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::loaders::action_entry_loader::{ACTION_ENTRY_LOADER_IDENTIFIER, ActionEntryLoader};

#[plugin]
#[derive(Debug)]
pub struct ActionEntryLoaderPlugin;

impl ActionEntryLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

impl Default for ActionEntryLoaderPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl Plugin for ActionEntryLoaderPlugin {
  fn name(&self) -> &'static str {
    "ActionEntryLoaderPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));
    Ok(())
  }
}

#[plugin_hook(NormalModuleFactoryResolveLoader for ActionEntryLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;
  if loader_request.starts_with(ACTION_ENTRY_LOADER_IDENTIFIER) {
    let loader = Arc::new(ActionEntryLoader::new().with_identifier(loader_request.clone()));
    return Ok(Some(loader));
  }
  Ok(None)
}
