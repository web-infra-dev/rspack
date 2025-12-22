use once_cell::sync::Lazy;
use rspack_collections::IdentifierSet;
use rspack_core::CompilerId;
use rspack_util::{atom::Atom, fx_hash::FxIndexSet};
use rustc_hash::FxHashMap;
use tokio::sync::Mutex;

use crate::reference_manifest::{ManifestExport, ModuleLoading, ServerReferenceManifest};

pub type ActionIdNamePair = (Atom, Atom);

#[derive(Debug, Default)]
pub struct PluginState {
  pub module_loading: Option<ModuleLoading>,
  pub injected_client_entries: FxHashMap<String, String>,
  pub client_modules: FxHashMap<String, ManifestExport>,
  pub ssr_modules: FxHashMap<String, ManifestExport>,
  pub client_actions_per_entry: FxHashMap<String, FxHashMap<String, Vec<ActionIdNamePair>>>,
  pub server_actions: ServerReferenceManifest,
  pub entry_css_imports: FxHashMap<String, FxHashMap<String, FxIndexSet<String>>>,
  pub entry_css_files: FxHashMap<String, FxHashMap<String, FxIndexSet<String>>>,
  /// Nested map of JS chunk files for each entry pair.
  ///
  /// Structure:
  /// - key: entry name
  /// - value: ordered set of JS chunk file paths (deduped)
  pub entry_js_files: FxHashMap<String, FxIndexSet<String>>,
  pub changed_server_components_per_entry: FxHashMap<String, IdentifierSet>,
}

impl PluginState {
  pub fn clear(&mut self) {
    self.module_loading = None;
    self.injected_client_entries.clear();
    self.client_modules.clear();
    self.ssr_modules.clear();
    self.client_actions_per_entry.clear();
    self.server_actions.clear();
    self.entry_css_imports.clear();
    self.entry_css_files.clear();
    self.entry_js_files.clear();
    self.changed_server_components_per_entry.clear();
  }
}

pub static PLUGIN_STATE_BY_COMPILER_ID: Lazy<Mutex<FxHashMap<CompilerId, PluginState>>> =
  Lazy::new(|| Default::default());
