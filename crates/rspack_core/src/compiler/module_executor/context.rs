use rspack_collections::IdentifierSet;
use rustc_hash::FxHashMap as HashMap;

use super::module_tracker::ModuleTracker;
use crate::{make::repair::MakeTaskContext, Context, DependencyId};

/// The meta data for import_module.
///
/// If the meta data is same, we can assume that it is a same entry.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ImportModuleMeta {
  pub origin_module_context: Context,
  pub request: String,
  pub layer: Option<String>,
}

/// The entry for import_module.
#[derive(Debug)]
pub struct ImportModuleEntry {
  /// The ImportModuleDependency dep_id.
  pub dep_id: DependencyId,
  /// The origin module identifiers which used this entry.
  pub origin_module_identifiers: IdentifierSet,
}

/// A task context for module executor.
#[derive(Debug)]
pub struct ExecutorTaskContext {
  /// The make task context.
  pub origin_context: MakeTaskContext,
  /// Module tracker.
  pub tracker: ModuleTracker,
  /// Entries.
  pub entries: HashMap<ImportModuleMeta, ImportModuleEntry>,
  /// The entry used during the current compilation.
  ///
  /// The key is entry dependency id, the value is the origin_module_identifier that triggers it.
  /// When the module executor stops, the entries whose all of origin_module_identifiers have been revoked will be deleted.
  pub used_entry: HashMap<DependencyId, IdentifierSet>,
}
