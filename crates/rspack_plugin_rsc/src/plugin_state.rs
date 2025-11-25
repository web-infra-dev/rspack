use once_cell::sync::Lazy;
use rspack_core::CompilerId;
use rustc_hash::FxHashMap;
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct PluginState {
  pub injected_client_entries: FxHashMap<String, String>,
}

pub static PLUGIN_STATE_BY_COMPILER_ID: Lazy<Mutex<FxHashMap<CompilerId, PluginState>>> =
  Lazy::new(|| Default::default());
