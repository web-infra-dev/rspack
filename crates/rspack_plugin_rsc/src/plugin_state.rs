use once_cell::sync::Lazy;
use rspack_core::CompilerId;
use rustc_hash::{FxHashMap, FxHashSet};
use tokio::sync::Mutex;

use crate::reference_manifest::{ManifestExport, ModuleLoading, ServerReferenceManifest};

#[derive(Debug, Default)]
pub struct PluginState {
  pub module_loading: Option<ModuleLoading>,
  pub injected_client_entries: FxHashMap<String, String>,
  pub client_modules: FxHashMap<String, ManifestExport>,
  pub ssr_modules: FxHashMap<String, ManifestExport>,
  pub server_actions: ServerReferenceManifest,
  pub entry_css_files: FxHashMap<String, FxHashSet<String>>,
}

pub static PLUGIN_STATE_BY_COMPILER_ID: Lazy<Mutex<FxHashMap<CompilerId, PluginState>>> =
  Lazy::new(|| Default::default());
