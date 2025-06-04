use rspack_error::Result;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::module_tracker::ModuleTracker;
use crate::{make::repair::MakeTaskContext, BoxModule, DependencyId, ModuleIdentifier};

#[allow(clippy::type_complexity)]
pub struct Callback(pub Box<dyn FnOnce(Result<&BoxModule>)>);

unsafe impl Send for Callback {}

impl std::fmt::Debug for Callback {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Callback").finish()
  }
}

/// The meta data for load_module.
///
/// If the meta data is same, we can assume that it is a same entry.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct LoadModuleMeta {
  pub origin_module_identifier: ModuleIdentifier,
  pub request: String,
  pub layer: Option<String>,
}

/// A task context for module executor.
#[derive(Debug)]
pub struct LoadTaskContext {
  /// The make task context.
  pub origin_context: MakeTaskContext,
  /// Entries.
  pub entries: HashMap<LoadModuleMeta, DependencyId>,
  pub tracker: ModuleTracker,
  /// The entry used during the current compilation.
  ///
  /// The key is entry dependency id, the value is the origin_module_identifier that triggers it.
  /// When the module executor stops, the entries whose all of origin_module_identifiers have been revoked will be deleted.
  pub used_entry: HashSet<DependencyId>,
}
