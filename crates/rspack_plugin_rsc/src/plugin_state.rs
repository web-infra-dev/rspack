use once_cell::sync::Lazy;
use rspack_core::CompilerId;
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::FxHashMap;
use tokio::sync::Mutex;

use crate::reference_manifest::{ManifestExport, ModuleLoading, ServerReferenceManifest};

#[derive(Debug, Default)]
pub struct PluginState {
  pub module_loading: Option<ModuleLoading>,
  pub injected_client_entries: FxHashMap<String, String>,
  pub client_modules: FxHashMap<String, ManifestExport>,
  pub ssr_modules: FxHashMap<String, ManifestExport>,
  pub server_actions: ServerReferenceManifest,
  // Key: server entry name
  // Value: CSS import specifiers/requests (e.g. resolved `path + query`)
  // TODO: 应该区分同一个 server entry 在不同 entry 的情况，
  // 例如 entry 具有不同的 layer，同一个 server entry 的表现将不同
  pub entry_css_imports: FxHashMap<String, FxIndexSet<String>>,
  pub entry_css_files: FxHashMap<String, FxIndexSet<String>>,
  /// Nested map of JS chunk files for each entry pair.
  ///
  /// Structure:
  /// - key: entry name
  /// - value: ordered set of JS chunk file paths (deduped)
  pub entry_js_files: FxHashMap<String, FxIndexSet<String>>,
}

pub static PLUGIN_STATE_BY_COMPILER_ID: Lazy<Mutex<FxHashMap<CompilerId, PluginState>>> =
  Lazy::new(|| Default::default());
