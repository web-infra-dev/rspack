use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::module_tracker::ModuleTracker;
use crate::{make::repair::MakeTaskContext, DependencyId, ModuleIdentifier};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ImportModuleMeta {
  // TODO remove option
  pub origin_module_identifier: Option<ModuleIdentifier>,
  pub request: String,
  pub layer: Option<String>,
}

#[derive(Debug)]
pub struct ExecutorTaskContext {
  pub origin_context: MakeTaskContext,
  pub tracker: ModuleTracker,
  pub entries: HashMap<ImportModuleMeta, DependencyId>,
  pub executed_entry_deps: HashSet<DependencyId>,
}
