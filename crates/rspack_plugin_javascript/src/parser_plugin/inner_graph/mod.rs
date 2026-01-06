use rspack_core::{
  ExportsInfoArtifact, ModuleGraph, ModuleGraphConnection, RuntimeSpec, UsageState, UsedByExports,
};

pub mod plugin;

pub fn connection_active_used_by_exports(
  connection: &ModuleGraphConnection,
  runtime: Option<&RuntimeSpec>,
  mg: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  used_by_exports: Option<&UsedByExports>,
) -> bool {
  let Some(used_by_exports) = used_by_exports.as_ref() else {
    return true;
  };
  let used_by_exports = match used_by_exports {
    UsedByExports::Set(used_by_exports) => used_by_exports,
    UsedByExports::Bool(used) => return *used,
  };
  let module_identifier = mg
    .get_parent_module(&connection.dependency_id)
    .expect("should have parent module");
  let exports_info = exports_info_artifact.get_exports_info_data(module_identifier);
  used_by_exports.iter().any(|name| {
    exports_info
      .named_exports(name)
      .unwrap_or_else(|| exports_info.other_exports_info())
      .get_used(runtime)
      != UsageState::Unused
  })
}
