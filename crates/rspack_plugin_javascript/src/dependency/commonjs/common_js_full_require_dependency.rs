use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, ConnectionState, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyCondition, DependencyConditionFn, DependencyId, DependencyLocation, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, ExportsInfoArtifact, ExportsType,
  ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, ModuleGraphConnection, ReferencedExport,
  RuntimeGlobals, RuntimeSpec, SideEffectsStateArtifact, TemplateContext, TemplateReplaceSource,
  UsedByExports, UsedName, create_exports_object_referenced, property_access, to_normal_comment,
};

use crate::{Atom, connection_active_used_by_exports};

#[cacheable]
#[derive(Debug)]
pub struct CommonJsFullRequireDependency {
  id: DependencyId,
  request: String,
  #[cacheable(with=AsVec<AsPreset>)]
  names: Vec<Atom>,
  range: DependencyRange,
  is_call: bool,
  namespace_object_as_context: bool,
  optional: bool,
  asi_safe: bool,
  loc: Option<DependencyLocation>,
  used_by_exports: Option<UsedByExports>,
}

impl CommonJsFullRequireDependency {
  #[allow(clippy::too_many_arguments)]
  #[allow(clippy::fn_params_excessive_bools)]
  pub fn new(
    request: String,
    names: Vec<Atom>,
    range: DependencyRange,
    loc: Option<DependencyLocation>,
    is_call: bool,
    namespace_object_as_context: bool,
    optional: bool,
    asi_safe: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      names,
      range,
      is_call,
      namespace_object_as_context,
      optional,
      asi_safe,
      loc,
      used_by_exports: None,
    }
  }

  pub fn set_used_by_exports(&mut self, used_by_exports: Option<UsedByExports>) {
    self.used_by_exports = used_by_exports;
  }
}

#[cacheable_dyn]
impl Dependency for CommonJsFullRequireDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsFullRequire
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn get_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ReferencedExport> {
    let namespace_object_as_context = self.namespace_object_as_context;

    let module = module_graph
      .get_module_by_dependency_id(&self.id)
      .expect("should have module");
    let exports_type = module.get_exports_type(
      module_graph,
      module_graph_cache,
      exports_info_artifact,
      false,
    );

    // CommonJS exports are real objects, so a member call can observe the whole
    // object through `this`. ESM namespace objects only need this bailout when
    // strictThisContextOnImports is enabled.
    if self.is_call
      && (namespace_object_as_context || !matches!(exports_type, ExportsType::Namespace))
    {
      if self.names.is_empty() {
        return create_exports_object_referenced();
      }
      return vec![ReferencedExport::from(
        &self.names[..self.names.len().saturating_sub(1)],
      )];
    }
    vec![ReferencedExport::from(self.names.as_slice())]
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for CommonJsFullRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    self
      .used_by_exports
      .is_some()
      .then(|| DependencyCondition::new(CommonJsFullRequireDependencyCondition))
  }
}

struct CommonJsFullRequireDependencyCondition;

impl DependencyConditionFn for CommonJsFullRequireDependencyCondition {
  fn get_connection_state(
    &self,
    connection: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState {
    let dependency = module_graph.dependency_by_id(&connection.dependency_id);
    let dependency = dependency
      .downcast_ref::<CommonJsFullRequireDependency>()
      .expect("should be CommonJsFullRequireDependency");
    ConnectionState::Active(connection_active_used_by_exports(
      connection,
      runtime,
      module_graph,
      exports_info_artifact,
      dependency.used_by_exports.as_ref(),
    ))
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CommonJsFullRequireDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CommonJsFullRequireDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CommonJsFullRequireDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CommonJsFullRequireDependencyTemplate;

impl CommonJsFullRequireDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CjsFullRequire)
  }
}

impl DependencyTemplate for CommonJsFullRequireDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CommonJsFullRequireDependency>()
      .expect("CommonJsFullRequireDependencyTemplate should only be used for CommonJsFullRequireDependency");

    let TemplateContext {
      compilation,
      runtime,
      runtime_template,
      ..
    } = code_generatable_context;
    let module_graph = compilation.get_module_graph();

    let require_expr = if let Some(imported_module) =
      module_graph.module_graph_module_by_dependency_id(&dep.id)
      && let Some(used) = {
        let exports_info = compilation
          .exports_info_artifact
          .get_exports_info_data(&imported_module.module_identifier);
        exports_info.get_used_name(&compilation.exports_info_artifact, *runtime, &dep.names)
      } {
      let mut require_expr = match used {
        UsedName::Normal(used) => {
          format!(
            "{}({}){}{}",
            runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
            runtime_template.module_id(compilation, &dep.id, &dep.request, false),
            to_normal_comment(&property_access(&dep.names, 0)),
            property_access(used, 0)
          )
        }
        UsedName::Inlined(inlined) => inlined.render(&to_normal_comment(&format!(
          "inlined export {}",
          property_access(&dep.names, 0)
        ))),
      };
      if dep.asi_safe {
        require_expr = format!("({require_expr})");
      }
      require_expr
    } else {
      format!(
        r#"{}({})"#,
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
        runtime_template.module_id(compilation, &dep.id, &dep.request, false)
      )
    };

    source.replace(dep.range.start, dep.range.end, require_expr, None);
  }
}
