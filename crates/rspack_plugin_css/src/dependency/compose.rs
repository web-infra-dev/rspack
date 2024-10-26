use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyRange, DependencyType, ExtendedReferencedExport, ModuleDependency, RuntimeSpec,
};
use rspack_util::atom::Atom;

#[derive(Debug, Clone)]
pub struct CssComposeDependency {
  id: DependencyId,
  request: String,
  name: Vec<Atom>,
  range: DependencyRange,
}

impl CssComposeDependency {
  pub fn new(request: String, name: Vec<Atom>, range: DependencyRange) -> Self {
    Self {
      id: DependencyId::new(),
      name,
      request,
      range,
    }
  }
}

impl Dependency for CssComposeDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssCompose
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssCompose
  }

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &rspack_core::ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<rspack_core::ExtendedReferencedExport> {
    vec![ExtendedReferencedExport::Array(self.name.clone())]
  }
}

impl ModuleDependency for CssComposeDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}

impl AsDependencyTemplate for CssComposeDependency {}
impl AsContextDependency for CssComposeDependency {}
