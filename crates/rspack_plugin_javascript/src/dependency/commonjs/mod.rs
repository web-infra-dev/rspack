mod common_js_dependency_helpers;
mod common_js_export_require_dependency;
mod common_js_exports_dependency;
mod common_js_full_require_dependency;
mod common_js_object_literal_export_dependency;
mod common_js_require_dependency;
mod common_js_self_reference_dependency;
mod module_decorator_dependency;
mod require_ensure_dependency;
mod require_ensure_item_dependency;
mod require_header_dependency;
mod require_main_dependency;
mod require_resolve_dependency;
mod require_resolve_header_dependency;

use std::sync::Arc;

pub use common_js_export_require_dependency::{
  CommonJsExportRequireDependency, CommonJsExportRequireDependencyTemplate,
};
pub use common_js_exports_dependency::{
  CommonJsExportsDependency, CommonJsExportsDependencyTemplate, ExportsBase,
};
pub use common_js_full_require_dependency::{
  CommonJsFullRequireDependency, CommonJsFullRequireDependencyTemplate,
};
pub use common_js_object_literal_export_dependency::{
  CommonJsObjectLiteralExportDependency, CommonJsObjectLiteralExportDependencyTemplate,
  CommonJsObjectLiteralExportKind,
};
pub use common_js_require_dependency::{
  CommonJsRequireDependency, CommonJsRequireDependencyTemplate,
};
pub use common_js_self_reference_dependency::{
  CommonJsSelfReferenceDependency, CommonJsSelfReferenceDependencyTemplate,
};
pub use module_decorator_dependency::{
  ModuleDecoratorDependency, ModuleDecoratorDependencyTemplate,
};
pub use require_ensure_dependency::{RequireEnsureDependency, RequireEnsureDependencyTemplate};
pub use require_ensure_item_dependency::RequireEnsureItemDependency;
pub use require_header_dependency::{RequireHeaderDependency, RequireHeaderDependencyTemplate};
pub use require_main_dependency::{RequireMainDependency, RequireMainDependencyTemplate};
pub use require_resolve_dependency::{RequireResolveDependency, RequireResolveDependencyTemplate};
pub use require_resolve_header_dependency::{
  RequireResolveHeaderDependency, RequireResolveHeaderDependencyTemplate,
};
use rspack_core::{
  ConnectionState, Context, DependencyCondition, DependencyConditionFn, ExportsInfoArtifact,
  ModuleGraph, ModuleGraphCacheArtifact, ModuleGraphConnection, RuntimeSpec,
  SideEffectsStateArtifact, UsedByExports,
};

use crate::connection_active_used_by_exports;

struct UsedByExportsDependencyCondition(Arc<UsedByExports>);

impl DependencyConditionFn for UsedByExportsDependencyCondition {
  fn get_connection_state(
    &self,
    connection: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState {
    ConnectionState::Active(connection_active_used_by_exports(
      connection,
      runtime,
      module_graph,
      exports_info_artifact,
      Some(&self.0),
    ))
  }
}

pub(super) fn used_by_exports_condition(
  used_by_exports: Option<&Arc<UsedByExports>>,
) -> Option<DependencyCondition> {
  used_by_exports.map(|used_by_exports| {
    DependencyCondition::new(UsedByExportsDependencyCondition(used_by_exports.clone()))
  })
}

pub(super) fn create_resource_identifier_for_contextual_commonjs_dependency(
  dep_type: &str,
  context: &Context,
  request: &str,
) -> String {
  let mut identifier = String::with_capacity(dep_type.len() + context.len() + request.len() + 2);
  identifier.push_str(dep_type);
  identifier.push('|');
  identifier.push_str(context);
  identifier.push('|');
  identifier.push_str(request);
  identifier
}

static OBJECT_PROTOTYPE_METHODS: &[&str] = &[
  "constructor",
  "__defineGetter__",
  "__defineSetter__",
  "hasOwnProperty",
  "__lookupGetter__",
  "__lookupSetter__",
  "isPrototypeOf",
  "propertyIsEnumerable",
  "toString",
  "valueOf",
  "__proto__",
  "toLocaleString",
];
