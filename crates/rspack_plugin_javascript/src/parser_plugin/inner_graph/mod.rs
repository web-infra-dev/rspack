use std::sync::Arc;

use rspack_core::{
  ConnectionState, DependencyCondition, DependencyConditionFn, DependencyId, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleGraphConnection, PrefetchExportsInfoMode,
  PrefetchedExportsInfoWrapper, RuntimeSpec, UsageState, UsedByExports,
};
use rspack_util::atom::Atom;
use rustc_hash::FxHashSet;

pub mod plugin;
pub mod state;

#[derive(Clone)]
struct UsedByExportsDependencyCondition {
  dependency_id: DependencyId,
  used_by_exports: FxHashSet<Atom>,
}

impl DependencyConditionFn for UsedByExportsDependencyCondition {
  fn get_connection_state(
    &self,
    _conn: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    mg: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> ConnectionState {
    let module_identifier = mg
      .get_parent_module(&self.dependency_id)
      .expect("should have parent module");
    let exports_info =
      mg.get_prefetched_exports_info(module_identifier, PrefetchExportsInfoMode::Default);
    ConnectionState::Active(is_connection_active(
      &exports_info,
      &self.used_by_exports,
      runtime,
    ))
  }
}

fn is_connection_active(
  exports_info: &PrefetchedExportsInfoWrapper<'_>,
  used_by_exports: &FxHashSet<Atom>,
  runtime: Option<&RuntimeSpec>,
) -> bool {
  fn is_export_active(
    name: &Atom,
    exports_info: &PrefetchedExportsInfoWrapper<'_>,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    if let Some(export_info) = exports_info.named_export(name) {
      return export_info.get_used(runtime) != UsageState::Unused;
    }
    exports_info.other_exports_info().get_used(runtime) != UsageState::Unused
  }

  used_by_exports
    .iter()
    .any(|name| is_export_active(name, exports_info, runtime))
}

// https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/InnerGraph.js#L319-L338
pub fn get_dependency_used_by_exports_condition(
  dependency_id: DependencyId,
  used_by_exports: Option<&UsedByExports>,
) -> Option<DependencyCondition> {
  match used_by_exports {
    Some(UsedByExports::Set(used_by_exports)) => Some(DependencyCondition::Fn(Arc::new(
      UsedByExportsDependencyCondition {
        dependency_id,
        used_by_exports: used_by_exports.clone(),
      },
    ))),
    Some(UsedByExports::Bool(bool)) => {
      if *bool {
        None
      } else {
        Some(DependencyCondition::False)
      }
    }
    None => None,
  }
}
