use rspack_core::{
  module_namespace_promise, AsContextDependency, Dependency, DependencyCategory, DependencyId,
  DependencyTemplate, DependencyType, ErrorSpan, ExtendedReferencedExport, ImportDependencyTrait,
  ModuleDependency, ModuleGraph, ReferencedExport, RuntimeSpec, TemplateContext,
  TemplateReplaceSource,
};
use swc_core::ecma::atoms::Atom;

#[derive(Debug, Clone)]
pub struct ImportEagerDependency {
  start: u32,
  end: u32,
  id: DependencyId,
  request: Atom,
  span: Option<ErrorSpan>,
  referenced_exports: Option<Vec<Atom>>,
}

impl ImportEagerDependency {
  pub fn new(
    start: u32,
    end: u32,
    request: Atom,
    span: Option<ErrorSpan>,
    referenced_exports: Option<Vec<Atom>>,
  ) -> Self {
    Self {
      start,
      end,
      request,
      span,
      id: DependencyId::new(),
      referenced_exports,
    }
  }
}

impl Dependency for ImportEagerDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::DynamicImportEager
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }

  fn dependency_debug_name(&self) -> &'static str {
    "ImportEagerDependency"
  }
}

impl ModuleDependency for ImportEagerDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    if let Some(referenced_exports) = &self.referenced_exports {
      vec![ReferencedExport::new(referenced_exports.clone(), false).into()]
    } else {
      vec![ExtendedReferencedExport::Array(vec![])]
    }
  }
}

impl ImportDependencyTrait for ImportEagerDependency {
  fn referenced_exports(&self) -> Option<&Vec<Atom>> {
    self.referenced_exports.as_ref()
  }
}

impl DependencyTemplate for ImportEagerDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let block = code_generatable_context
      .compilation
      .get_module_graph()
      .get_parent_block(&self.id);
    source.replace(
      self.start,
      self.end,
      module_namespace_promise(
        code_generatable_context,
        &self.id,
        block,
        &self.request,
        self.dependency_type().as_str().as_ref(),
        false,
      )
      .as_str(),
      None,
    );
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsContextDependency for ImportEagerDependency {}
