use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, Context, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyId, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExportsInfoArtifact, FactorizeInfo, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  ReferencedExport, ResourceIdentifier, RuntimeSpec, TemplateContext, TemplateReplaceSource,
};

use super::create_resource_identifier_for_contextual_commonjs_dependency;

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireResolveDependency {
  pub id: DependencyId,
  pub request: String,
  pub weak: bool,
  range: DependencyRange,
  namespace_object_mode_range: Option<DependencyRange>,
  optional: bool,
  context: Option<Context>,
  resource_identifier: ResourceIdentifier,
  factorize_info: FactorizeInfo,
}

impl RequireResolveDependency {
  pub fn new(request: String, range: DependencyRange, weak: bool, optional: bool) -> Self {
    Self {
      range,
      request,
      weak,
      optional,
      id: DependencyId::new(),
      namespace_object_mode_range: None,
      context: None,
      resource_identifier: Default::default(),
      factorize_info: Default::default(),
    }
  }

  pub fn new_for_namespace_object(
    request: String,
    range: DependencyRange,
    weak: bool,
    optional: bool,
    namespace_object_mode_range: DependencyRange,
  ) -> Self {
    Self {
      namespace_object_mode_range: Some(namespace_object_mode_range),
      ..Self::new(request, range, weak, optional)
    }
  }

  pub fn new_contextual(
    request: String,
    range: DependencyRange,
    weak: bool,
    optional: bool,
    context: Context,
  ) -> Self {
    let resource_identifier = create_resource_identifier_for_contextual_commonjs_dependency(
      "require.resolve",
      &context,
      &request,
    )
    .into();
    Self {
      context: Some(context),
      resource_identifier,
      ..Self::new(request, range, weak, optional)
    }
  }
}

#[cacheable_dyn]
impl Dependency for RequireResolveDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireResolve
  }

  fn get_context(&self) -> Option<&Context> {
    self.context.as_ref()
  }

  fn resource_identifier(&self) -> Option<&str> {
    self
      .context
      .as_ref()
      .map(|_| self.resource_identifier.as_str())
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ReferencedExport> {
    vec![]
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
impl ModuleDependency for RequireResolveDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn weak(&self) -> bool {
    self.weak
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for RequireResolveDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RequireResolveDependencyTemplate::template_type())
  }
}

impl AsContextDependency for RequireResolveDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RequireResolveDependencyTemplate;

impl RequireResolveDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::RequireResolve)
  }
}

impl DependencyTemplate for RequireResolveDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RequireResolveDependency>()
      .expect("RequireResolveDependencyTemplate should only be used for RequireResolveDependency");

    if let Some(mode_range) = dep.namespace_object_mode_range
      && code_generatable_context.is_modern_module_output()
    {
      let kind = if dep.weak {
        rspack_core::CodeGenerationModuleReferenceKind::WeakValue
      } else {
        rspack_core::CodeGenerationModuleReferenceKind::Value
      };
      let Some(module_value) = code_generatable_context.create_module_relocation(dep.id, kind)
      else {
        source.replace(
          dep.range.start,
          dep.range.end,
          code_generatable_context
            .runtime_template
            .missing_module(&dep.request),
          None,
        );
        return;
      };
      let module_value = if dep.weak {
        format!(
          "(({module_value}) || {})()",
          code_generatable_context
            .runtime_template
            .weak_error_function(&dep.request)
        )
      } else {
        module_value
      };
      source.replace(dep.range.start, dep.range.end, module_value, None);
      source.insert_static(mode_range.start, "(", None);
      source.insert_static(mode_range.end, ") & ~1", None);
      return;
    }

    source.replace(
      dep.range.start,
      dep.range.end,
      code_generatable_context.runtime_template.module_id(
        code_generatable_context.compilation,
        &dep.id,
        &dep.request,
        dep.weak,
      ),
      None,
    );
  }
}
