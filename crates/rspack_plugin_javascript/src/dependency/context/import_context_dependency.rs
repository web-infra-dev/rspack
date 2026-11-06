use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_collections::Identifier;
use rspack_core::{
  AsModuleDependency, ContextDependency, ContextOptions, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportsInfoArtifact, ImportAttributes, ModuleGraph,
  ModuleGraphCacheArtifact, ReferencedSpecifier, ResourceIdentifier, TemplateContext,
  TemplateReplaceSource,
};
use rspack_error::Diagnostic;
use rspack_util::json_stringify;

use super::{
  context_dependency_template_as_require_call, create_resource_identifier_and_regex_ranges,
};

fn create_resource_identifier_and_range(
  options: &ContextOptions,
) -> (Identifier, Vec<std::ops::Range<usize>>) {
  let (resource_identifier, range) = create_resource_identifier_and_regex_ranges(None, options);
  let Some(attributes) = &options.attributes else {
    return (resource_identifier, range);
  };
  let mut resource_identifier = resource_identifier.to_string();
  resource_identifier.push_str(&json_stringify(attributes));
  (resource_identifier.into(), range)
}

fn create_resource_identifier(options: &ContextOptions) -> Identifier {
  create_resource_identifier_and_range(options).0
}

#[cacheable]
#[derive(Debug)]
pub struct ImportContextDependency {
  id: DependencyId,
  options: ContextOptions,
  range: DependencyRange,
  value_range: DependencyRange,
  resource_identifier: ResourceIdentifier,
  optional: bool,
  critical: rspack_core::DependencyCriticalState,
}

impl ImportContextDependency {
  pub fn new(
    options: ContextOptions,
    range: DependencyRange,
    value_range: DependencyRange,
    optional: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      resource_identifier: create_resource_identifier(&options),
      range,
      value_range,
      optional,
      critical: Default::default(),
      options,
    }
  }

  pub fn set_referenced_specifiers(
    &mut self,
    referenced_specifiers: Vec<ReferencedSpecifier>,
    from_magic_comment: bool,
  ) {
    if !from_magic_comment && referenced_specifiers.is_empty() {
      // If the referenced specifiers are empty, keep it as default (None), since this dependency can't eliminate by side effects optimization,
      // so if we set it to Some(vec![]), and the dependency still executes, it will cause runtime error because the exports are all tree shaken.
      // see test case `tests/rspack-test/configCases/tree-shaking/side-effects-free-dynamic-import`
      return;
    }
    self.options.referenced_specifiers = Some(referenced_specifiers);
    self.resource_identifier = create_resource_identifier(&self.options);
  }
}

#[cacheable_dyn]
impl Dependency for ImportContextDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportContext
  }

  fn get_phase(&self) -> rspack_core::ImportPhase {
    self.options.phase.unwrap_or_default()
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    self.options.attributes.as_ref()
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }

  fn get_diagnostics(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<Vec<Diagnostic>> {
    if let Some(critical) = self.critical() {
      return Some(vec![critical]);
    }
    None
  }
}

impl ContextDependency for ImportContextDependency {
  fn options(&self) -> &ContextOptions {
    &self.options
  }

  fn request(&self) -> &str {
    &self.options.request
  }

  fn get_context(&self) -> Option<&str> {
    None
  }

  fn resource_identifier(&self) -> &str {
    &self.resource_identifier
  }

  fn resource_identifier_regex_ranges(&self) -> Vec<std::ops::Range<usize>> {
    let (identifier, ranges) = super::create_resource_identifier_and_regex_ranges(
      ContextDependency::get_context(self),
      self.options(),
    );
    if ContextDependency::resource_identifier(self).starts_with(identifier.as_str()) {
      ranges
    } else {
      Vec::new()
    }
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn type_prefix(&self) -> rspack_core::ContextTypePrefix {
    rspack_core::ContextTypePrefix::Import
  }

  fn critical(&self) -> Option<Diagnostic> {
    self.critical.get()
  }

  fn set_critical(&self, critical: Option<Diagnostic>) {
    self.critical.set(critical);
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ImportContextDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportContextDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for ImportContextDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportContextDependencyTemplate;

impl ImportContextDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::ImportContext)
  }
}

impl DependencyTemplate for ImportContextDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportContextDependency>()
      .expect("ImportContextDependencyTemplate should be used for ImportContextDependency");

    context_dependency_template_as_require_call(
      dep,
      source,
      code_generatable_context,
      &dep.range,
      Some(&dep.value_range),
    );
  }
}
