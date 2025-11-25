use once_cell::sync::Lazy;
use rspack_core::{CompilerId, ModuleId};
use rustc_hash::FxHashMap;
use tokio::sync::Mutex;

use crate::client_reference_manifest::ManifestExport;

#[derive(Debug, Default)]
pub struct PluginState {
  pub injected_client_entries: FxHashMap<String, String>,
  pub client_modules: FxHashMap<String, ManifestExport>,
  pub ssr_modules: FxHashMap<String, ManifestExport>,
}

pub static PLUGIN_STATE_BY_COMPILER_ID: Lazy<Mutex<FxHashMap<CompilerId, PluginState>>> =
  Lazy::new(|| Default::default());
