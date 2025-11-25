use std::sync::Arc;

use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_core::{
  BoxLoader, Compilation, CompilerCompilationHook, CompilerId, CompilerMake, Context,
  EntryDependency, EntryOptions, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin,
  Resolver,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};

use crate::{
  loaders::client_entry_loader::{CLIENT_ENTRY_LOADER_IDENTIFIER, ClientEntryLoader},
  plugin_state::PLUGIN_STATE_BY_COMPILER_ID,
};

type GetServerCompilerId = Box<dyn Fn() -> BoxFuture<'static, Result<CompilerId>> + Sync + Send>;

pub struct ReactClientPluginOptions {
  pub get_server_compiler_id: GetServerCompilerId,
}

#[plugin]
#[derive(Debug)]
pub struct ReactClientPlugin {
  #[debug(skip)]
  get_server_compiler_id: GetServerCompilerId,
}

impl ReactClientPlugin {
  pub fn new(options: ReactClientPluginOptions) -> Self {
    Self::new_inner(options.get_server_compiler_id)
  }
}

impl Plugin for ReactClientPlugin {
  fn name(&self) -> &'static str {
    "ReactClientPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.make.tap(make::new(self));
    Ok(())
  }
}

#[plugin_hook(CompilerMake for ReactClientPlugin)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  let server_compiler_id = (self.get_server_compiler_id)().await?;

  let guard = PLUGIN_STATE_BY_COMPILER_ID.lock().await;
  let Some(plugin_state) = guard.get(&server_compiler_id) else {
    return Err(rspack_error::error!(
      "Failed to find plugin state for server compiler (ID: {}). \
     The server compiler may not have properly collected client entry information, \
     or the compiler has not been initialized yet.",
      server_compiler_id.as_u32()
    ));
  };

  let context = compilation.options.context.clone();
  for (runtime, import) in &plugin_state.injected_client_entries {
    let dependency = Box::new(EntryDependency::new(
      import.to_string(),
      context.clone(),
      Some("react-client-components".to_string()),
      false,
    ));
    compilation
      .add_entry(
        dependency,
        EntryOptions {
          name: Some(format!("{}_client-components", runtime)),
          runtime: Some(runtime.to_string().into()),
          layer: Some("react-client-components".to_string()),
          ..Default::default()
        },
      )
      .await?;
  }

  Ok(())
}
