use std::sync::Arc;

use once_cell::sync::Lazy;
use rspack_cacheable::with::{AsPreset, AsVec};
use rspack_collections::IdentifierSet;
use rspack_core::CompilerId;
use rspack_util::{
  atom::Atom,
  fx_hash::{FxDashMap, FxIndexSet},
};
use rustc_hash::FxHashMap;

use crate::reference_manifest::{
  ManifestExport, ManifestNode, ModuleLoading, ServerReferenceManifest,
};

pub type ActionIdNamePair = (Atom, Atom);
pub type CssImportsByServerEntry = FxHashMap<String, FxIndexSet<String>>;
pub type ClientModulesByServerEntry = FxHashMap<String, Vec<ClientModuleImport>>;
pub type RootCssImports = FxIndexSet<String>;

/// Structured info about a client module to inject into the client compiler.
#[rspack_cacheable::cacheable]
#[derive(Debug, Clone)]
pub struct ClientModuleImport {
  pub request: String,
  #[cacheable(with=AsVec<AsPreset>)]
  pub ids: FxIndexSet<Atom>,
}

#[derive(Debug, Default)]
pub struct ServerEntryState {
  /// CSS import paths referenced by this server entry.
  pub css_imports: FxIndexSet<String>,
  /// CSS chunk file paths emitted for this server entry.
  pub css_files: FxIndexSet<String>,
}

/// State for one compilation entry.
#[derive(Debug, Default)]
pub struct EntryState {
  pub server_entries: FxHashMap<String, ServerEntryState>,
  /// All client modules discovered from the RSC graph for this entry.
  pub injected_client_entries: Vec<ClientModuleImport>,
  /// Client modules used by multiple owners and kept as individual async chunks.
  pub isolated_client_entries: Vec<ClientModuleImport>,
  /// Client modules used only by the root RSC tree.
  pub root_client_entries: Vec<ClientModuleImport>,
  /// Client modules used only by one `use server-entry` subtree.
  pub client_entries_by_server_entry: ClientModulesByServerEntry,
  pub client_modules: FxHashMap<String, ManifestExport>,
  /// Root CSS import paths reached through a parent chain without `use server-entry`.
  /// These are attached directly to the matching client compiler entry.
  pub root_css_imports: RootCssImports,
  /// Dependency path -> action id/name pairs.
  pub client_actions: FxHashMap<String, Vec<ActionIdNamePair>>,
  pub server_actions: ServerReferenceManifest,
  pub bootstrap_scripts: FxIndexSet<String>,
  pub changed_server_components: IdentifierSet,
  /// Precomputed in chunk_ids hook.
  pub server_consumer_module_map: Option<FxHashMap<String, ManifestNode>>,
}

impl EntryState {
  pub fn has_css_imports_by_server_entry(&self) -> bool {
    self
      .server_entries
      .values()
      .any(|server_entry| !server_entry.css_imports.is_empty())
  }

  pub fn css_imports_by_server_entry(&self) -> CssImportsByServerEntry {
    self
      .server_entries
      .iter()
      .filter(|(_, server_entry)| !server_entry.css_imports.is_empty())
      .map(|(name, server_entry)| (name.clone(), server_entry.css_imports.clone()))
      .collect()
  }
}

#[derive(Debug, Default)]
pub struct PluginState {
  pub module_loading: Option<ModuleLoading>,
  pub entries: FxHashMap<Arc<str>, EntryState>,
}

impl PluginState {
  pub fn clear(&mut self) {
    self.module_loading = None;
    self.entries.clear();
  }
}

pub static PLUGIN_STATES: Lazy<FxDashMap<CompilerId, PluginState>> = Lazy::new(Default::default);
