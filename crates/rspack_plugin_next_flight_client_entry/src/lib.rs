use std::collections::HashMap;

use async_trait::async_trait;
use futures::future::BoxFuture;
use rspack_core::{
  ApplyContext, Compilation, CompilerAfterEmit, CompilerFinishMake, CompilerOptions, Plugin,
  PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;

pub struct Worker {
  pub module_id: String,
  pub is_async: bool,
}

pub struct Action {
  pub workers: HashMap<String, Worker>,
  pub layer: HashMap<String, String>,
}

type Actions = HashMap<String, Action>;

pub struct ModuleInfo {
  pub module_id: String,
  pub is_async: bool,
}

pub struct ModulePair {
  pub server: Option<ModuleInfo>,
  pub client: Option<ModuleInfo>,
}

pub struct State {
  // A map to track "action" -> "list of bundles".
  pub server_actions: Actions,
  pub edge_server_actions: Actions,

  pub server_action_modules: HashMap<String, ModulePair>,
  pub edge_server_action_modules: HashMap<String, ModulePair>,

  pub ssr_modules: HashMap<String, ModuleInfo>,
  pub edge_ssr_modules: HashMap<String, ModuleInfo>,

  pub rsc_modules: HashMap<String, ModuleInfo>,
  pub edge_rsc_modules: HashMap<String, ModuleInfo>,

  pub injected_client_entries: HashMap<String, String>,
}

pub type StateCb = Box<dyn Fn(State) -> BoxFuture<'static, Result<()>> + Sync + Send>;

pub struct Options {
  pub dev: bool,
  pub app_dir: Utf8PathBuf,
  pub is_edge_server: bool,
  pub encryption_key: String,
  pub state_cb: StateCb,
}

#[plugin]
#[derive(Debug)]
pub struct FlightClientEntryPlugin {}

impl FlightClientEntryPlugin {
  pub fn new(options: Options) -> Self {
    Self::new_inner()
  }
}

#[plugin_hook(CompilerFinishMake for FlightClientEntryPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  Ok(())
}

// Next.js uses the after compile hook, but after emit should achieve the same result
#[plugin_hook(CompilerAfterEmit for FlightClientEntryPlugin)]
async fn after_emit(&self, compilation: &mut Compilation) -> Result<()> {
  Ok(())
}

#[async_trait]
impl Plugin for FlightClientEntryPlugin {
  fn name(&self) -> &'static str {
    "rspack.FlightClientEntryPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .finish_make
      .tap(finish_make::new(self));

    ctx
      .context
      .compiler_hooks
      .after_emit
      .tap(after_emit::new(self));

    Ok(())
  }
}
