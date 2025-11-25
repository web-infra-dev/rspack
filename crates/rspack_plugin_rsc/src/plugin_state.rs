use once_cell::sync::Lazy;
use rspack_core::{CompilerId, ModuleId};
use rustc_hash::FxHashMap;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ModuleInfo {
  pub module_id: ModuleId,
  pub r#async: bool,
}

#[derive(Debug, Default)]
pub struct PluginState {
  pub injected_client_entries: FxHashMap<String, String>,
  pub rsc_modules: FxHashMap<String, ModuleInfo>,
}

pub static PLUGIN_STATE_BY_COMPILER_ID: Lazy<Mutex<FxHashMap<CompilerId, PluginState>>> =
  Lazy::new(|| Default::default());
