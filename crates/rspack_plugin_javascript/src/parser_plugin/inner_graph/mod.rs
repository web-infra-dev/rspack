use std::sync::Arc;

use rspack_core::{
  ConnectionState, DependencyCondition, DependencyConditionFn, DependencyId, ExportInfoGetter,
  ExportsInfo, ModuleGraph, ModuleGraphCacheArtifact, ModuleGraphConnection, RuntimeSpec,
  UsageState, UsedByExports,
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
    ConnectionState::Active(is_connection_active(
      mg.get_exports_info(module_identifier),
      &self.used_by_exports,
      mg,
      runtime,
    ))
  }
}

fn is_connection_active(
  exports_info: ExportsInfo,
  used_by_exports: &FxHashSet<Atom>,
  mg: &ModuleGraph,
  runtime: Option<&RuntimeSpec>,
) -> bool {
  fn is_export_active(
    name: &Atom,
    exports_info: &ExportsInfo,
    mg: &ModuleGraph,
    runtime: Option<&RuntimeSpec>,
  ) -> bool {
    let exports_info = exports_info.as_data(mg);
    if let Some(export_info) = exports_info.named_exports(name) {
      let export_info = export_info.as_data(mg);
      return ExportInfoGetter::get_used(export_info, runtime) != UsageState::Unused;
    }
    if let Some(redirect_to) = exports_info.redirect_to {
      is_export_active(name, &redirect_to, mg, runtime)
    } else {
      ExportInfoGetter::get_used(exports_info.other_exports_info.as_data(mg), runtime)
        != UsageState::Unused
    }
  }

  used_by_exports
    .iter()
    .any(|name| is_export_active(name, &exports_info, mg, runtime))
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
