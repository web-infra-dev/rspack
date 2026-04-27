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
pub type CssImportsPerServerEntry = FxHashMap<String, FxIndexSet<String>>;
pub type RootCssImports = FxIndexSet<String>;

/// Structured info about a client module to inject into the client compiler.
#[rspack_cacheable::cacheable]
#[derive(Debug, Clone)]
pub struct ClientModuleImport {
  pub request: String,
  #[cacheable(with=AsVec<AsPreset>)]
  pub ids: FxIndexSet<Atom>,
}

/// State for one compilation entry.
#[derive(Debug, Default)]
pub struct EntryState {
  pub injected_client_entries: Vec<ClientModuleImport>,
  pub client_modules: FxHashMap<String, ManifestExport>,
  /// Server entry resource -> CSS import paths.
  pub css_imports_per_server_entry: CssImportsPerServerEntry,
  /// Root CSS import paths reached through a parent chain without `use server-entry`.
  /// These are attached directly to the matching client compiler entry.
  pub root_css_imports: RootCssImports,
  /// Dependency path -> action id/name pairs.
  pub client_actions: FxHashMap<String, Vec<ActionIdNamePair>>,
  pub server_actions: ServerReferenceManifest,
  /// Server entry resource -> CSS chunk file paths.
  pub entry_css_files: FxHashMap<String, FxIndexSet<String>>,
  pub entry_js_files: FxIndexSet<String>,
  pub changed_server_components: IdentifierSet,
  /// Precomputed in chunk_ids hook.
  pub server_consumer_module_map: Option<FxHashMap<String, ManifestNode>>,
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
