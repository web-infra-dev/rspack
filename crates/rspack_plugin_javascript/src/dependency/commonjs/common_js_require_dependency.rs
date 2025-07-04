use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  module_id, AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyId, DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType,
  DependencyType, FactorizeInfo, ModuleDependency, ModuleGraph, ModuleIdentifier, ModuleType,
  SharedSourceMap, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct CommonJsRequireDependency {
  id: DependencyId,
  request: String,
  optional: bool,
  range: DependencyRange,
  range_expr: Option<DependencyRange>,
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>,
  factorize_info: FactorizeInfo,
}

impl CommonJsRequireDependency {
  pub fn new(
    request: String,
    range: DependencyRange,
    range_expr: Option<DependencyRange>,
    optional: bool,
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      optional,
      range,
      range_expr,
      source_map,
      factorize_info: Default::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for CommonJsRequireDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.range.to_loc(self.source_map.as_ref())
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsRequire
  }

  fn range(&self) -> Option<&DependencyRange> {
    self.range_expr.as_ref()
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for CommonJsRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CommonJsRequireDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CommonJsRequireDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CommonJsRequireDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CommonJsRequireDependencyTemplate;

impl CommonJsRequireDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CjsRequire)
  }

  /// Enhanced ConsumeShared detection for CommonJS require dependencies
  fn detect_consume_shared_context(
    module_graph: &ModuleGraph,
    dep_id: &DependencyId,
    module_identifier: &ModuleIdentifier,
    _request: &str,
  ) -> Option<String> {
    // Check direct parent module for ConsumeShared context
    if let Some(parent_module_id) = module_graph.get_parent_module(dep_id) {
      if let Some(parent_module) = module_graph.module_by_identifier(parent_module_id) {
        if parent_module.module_type() == &ModuleType::ConsumeShared {
          return parent_module.get_consume_shared_key();
        }
      }
    }

    // Check incoming connections for ConsumeShared modules (fallback detection)
    for connection in module_graph.get_incoming_connections(module_identifier) {
      if let Some(origin_module) = connection.original_module_identifier.as_ref() {
        if let Some(origin_module_obj) = module_graph.module_by_identifier(origin_module) {
          if origin_module_obj.module_type() == &ModuleType::ConsumeShared {
            return origin_module_obj.get_consume_shared_key();
          }
        }
      }
    }
    // TODO: Implement proper ConsumeShared detection for CommonJS modules
    // Currently CommonJS modules accessed via require() cannot be made ConsumeShared
    // This is an architectural limitation that would require changes to the module resolution system

    None
  }
}

impl DependencyTemplate for CommonJsRequireDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CommonJsRequireDependency>()
      .expect(
        "CommonJsRequireDependencyTemplate should only be used for CommonJsRequireDependency",
      );

    // Get target module identifier for ConsumeShared detection
    let module_graph = &code_generatable_context.compilation.get_module_graph();
    let module_identifier = module_graph
      .module_identifier_by_dependency_id(&dep.id)
      .copied();

    // Detect ConsumeShared context
    let consume_shared_info = if let Some(target_module_id) = module_identifier {
      Self::detect_consume_shared_context(module_graph, &dep.id, &target_module_id, &dep.request)
    } else {
      None
    };

    // Generate base module reference
    let base_module_reference = module_id(
      code_generatable_context.compilation,
      &dep.id,
      &dep.request,
      false,
    );

    // Generate final replacement with ConsumeShared macro if applicable
    let final_replacement = if let Some(share_key) = consume_shared_info {
      format!(
        "/* @common:if [condition=\"treeShake.{share_key}.default\"] */ {base_module_reference} /* @common:endif */"
      )
    } else {
      base_module_reference.to_string()
    };

    source.replace(dep.range.start, dep.range.end - 1, &final_replacement, None);
  }
}
