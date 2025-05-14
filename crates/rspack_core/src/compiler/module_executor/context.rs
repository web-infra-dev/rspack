use rustc_hash::FxHashMap as HashMap;

use super::module_tracker::ModuleTracker;
use crate::{make::repair::MakeTaskContext, DependencyId, ModuleIdentifier};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ImportModuleMeta {
  // TODO remove option
  pub origin_module_identifier: Option<ModuleIdentifier>,
  pub request: String,
  pub layer: Option<String>,
}

pub struct ExecutorTaskContext {
  pub origin_context: MakeTaskContext,
  pub tracker: ModuleTracker,
  pub entries: HashMap<ImportModuleMeta, DependencyId>,
}
