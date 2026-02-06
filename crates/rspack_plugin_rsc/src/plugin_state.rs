use dashmap::DashMap;
use once_cell::sync::Lazy;
use rspack_collections::IdentifierSet;
use rspack_core::CompilerId;
use rspack_util::{atom::Atom, fx_hash::FxIndexSet};
use rustc_hash::FxHashMap;

use crate::reference_manifest::{ManifestExport, ModuleLoading, ServerReferenceManifest};

pub(crate) type ActionIdNamePair = (Atom, Atom);

#[derive(Debug, Default)]
pub(crate) struct PluginState {
  pub module_loading: Option<ModuleLoading>,
  pub injected_client_entries: FxHashMap<String, String>,
  pub client_modules: FxHashMap<String, ManifestExport>,
  pub ssr_modules: FxHashMap<String, ManifestExport>,
  pub client_actions_per_entry: FxHashMap<String, FxHashMap<String, Vec<ActionIdNamePair>>>,
  pub server_actions: ServerReferenceManifest,
  pub entry_css_imports: FxHashMap<String, FxHashMap<String, FxIndexSet<String>>>,
  /// Maps entry names to CSS chunk files organized by server entry resource.
  ///
  /// This nested structure tracks CSS dependencies for React Server Components:
  /// - Outer key: Entry name (e.g., "main", "app")
  /// - Inner key: Server entry resource
  /// - Inner value: Ordered set of CSS chunk file paths (automatically deduplicated)
  pub entry_css_files: FxHashMap<String, FxHashMap<String, FxIndexSet<String>>>,
  /// Maps entry names to their associated JS chunk files.
  ///
  /// This structure tracks JavaScript dependencies for React Server Components:
  /// - Key: Entry name (e.g., "main", "app")
  /// - Value: Ordered set of JS chunk file paths (automatically deduplicated)
  pub entry_js_files: FxHashMap<String, FxIndexSet<String>>,
  pub changed_server_components_per_entry: FxHashMap<String, IdentifierSet>,
}

impl PluginState {
  pub(crate) fn clear(&mut self) {
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

pub(crate) static PLUGIN_STATES: Lazy<DashMap<CompilerId, PluginState>> =
  Lazy::new(Default::default);
