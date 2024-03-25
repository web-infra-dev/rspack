use rspack_core::{module_raw, parse_resource, AsModuleDependency, ContextDependency};
use rspack_core::{normalize_context, DependencyCategory, DependencyId, DependencyTemplate};
use rspack_core::{ContextOptions, Dependency, TemplateReplaceSource};
use rspack_core::{DependencyType, ErrorSpan, TemplateContext};

use super::create_resource_identifier_for_context_dependency;

#[derive(Debug, Clone)]
pub struct ImportContextDependency {
  callee_start: u32,
  callee_end: u32,
  args_end: u32,
  id: DependencyId,
  options: ContextOptions,
  span: Option<ErrorSpan>,
  resource_identifier: String,
}

impl ImportContextDependency {
  pub fn new(
    callee_start: u32,
    callee_end: u32,
    args_end: u32,
    options: ContextOptions,
    span: Option<ErrorSpan>,
  ) -> Self {
    let resource_identifier = create_resource_identifier_for_context_dependency(None, &options);
    Self {
      callee_start,
      callee_end,
      args_end,
      options,
      span,
      id: DependencyId::new(),
      resource_identifier,
    }
  }
}

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

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }

  fn dependency_debug_name(&self) -> &'static str {
    "ImportContextDependency"
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

  fn set_request(&mut self, request: String) {
    self.options.request = request;
  }
}

impl DependencyTemplate for ImportContextDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      runtime_requirements,
      ..
    } = code_generatable_context;

    let expr = module_raw(
      compilation,
      runtime_requirements,
      &self.id,
      self.request(),
      false,
    );

    if compilation
      .get_module_graph()
      .module_graph_module_by_dependency_id(&self.id)
      .is_none()
    {
      source.replace(self.callee_start, self.args_end, &expr, None);
      return;
    }

    source.replace(self.callee_start, self.callee_end, &expr, None);

    let context = normalize_context(&self.options.context);
    let query = parse_resource(&self.options.request).and_then(|data| data.query);
    if !context.is_empty() || query.is_some() {
      source.insert(self.callee_end, "(", None);
      if !context.is_empty() {
        source.insert(
          self.args_end,
          format!(".replace('{context}', './')").as_str(),
          None,
        );
      }
      if let Some(query) = query {
        source.insert(
          self.args_end,
          format!(".replace('{query}', '')").as_str(),
          None,
        );
      }
      source.insert(self.args_end, ")", None);
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsModuleDependency for ImportContextDependency {}
