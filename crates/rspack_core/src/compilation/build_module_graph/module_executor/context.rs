use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{super::graph_updater::repair::context::TaskContext, module_tracker::ModuleTracker};
use crate::{DependencyId, ExportsInfoArtifact, ModuleIdentifier};

/// The meta data for import_module.
///
/// If the meta data is same, we can assume that it is a same entry.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ImportModuleMeta {
  pub origin_module_identifier: ModuleIdentifier,
  pub request: String,
  pub layer: Option<String>,
}

/// A task context for module executor.
#[derive(Debug)]
pub struct ExecutorTaskContext {
  /// The make task context.
  pub origin_context: TaskContext,
  /// Exports info artifact for module executor's isolated compilation environment.
  pub exports_info_artifact: ExportsInfoArtifact,
  /// module tracker.
  pub tracker: ModuleTracker,
  /// entries.
  ///
  /// All of the import module entry and their dependency id.
  pub entries: HashMap<ImportModuleMeta, DependencyId>,
  /// The entry deps used during the current compilation.
  ///
  /// When Module Executor stops, entries that are not in use
  /// and whose origin_module_identifier has been revoked are removed.
  pub executed_entry_deps: HashSet<DependencyId>,
}
