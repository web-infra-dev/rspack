use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCodeGeneration, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, RuntimeGlobals,
  TemplateContext, TemplateReplaceSource,
};

use super::CommonJsRequireDependency;

#[cacheable]
#[derive(Debug)]
pub struct RequireHeaderDependency {
  id: DependencyId,
  range: DependencyRange,
  loc: Option<DependencyLocation>,
  require_dependency_id: Option<DependencyId>,
}

impl RequireHeaderDependency {
  pub fn new(range: DependencyRange, loc: Option<DependencyLocation>) -> Self {
    Self::new_with_require_dependency(range, loc, None)
  }

  pub fn new_with_require_dependency(
    range: DependencyRange,
    loc: Option<DependencyLocation>,
    require_dependency_id: Option<DependencyId>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      loc,
      require_dependency_id,
    }
  }
}

#[cacheable_dyn]
impl Dependency for RequireHeaderDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for RequireHeaderDependency {}
impl AsContextDependency for RequireHeaderDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for RequireHeaderDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RequireHeaderDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RequireHeaderDependencyTemplate;

impl RequireHeaderDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("RequireHeaderDependency")
  }
}

impl DependencyTemplate for RequireHeaderDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RequireHeaderDependency>()
      .expect("RequireHeaderDependencyTemplate should only be used for RequireHeaderDependency");

    let TemplateContext {
      compilation,
      runtime,
      runtime_template,
      ..
    } = code_generatable_context;

    if let Some(require_dependency_id) = &dep.require_dependency_id {
      let module_graph = compilation.get_module_graph();
      let require_dependency = module_graph
        .dependency_by_id(require_dependency_id)
        .downcast_ref::<CommonJsRequireDependency>()
        .expect("paired dependency should be CommonJsRequireDependency");
      if require_dependency.is_evaluation_only()
        && let Some(connection) = module_graph.connection_by_dependency_id(require_dependency_id)
        && !connection.is_target_active(
          module_graph,
          *runtime,
          &compilation.module_graph_cache_artifact,
          &compilation
            .build_module_graph_artifact
            .side_effects_state_artifact,
          &compilation.exports_info_artifact,
        )
      {
        return;
      }
    }

    source.replace(
      dep.range.start,
      dep.range.end,
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
      None,
    );
  }
}
