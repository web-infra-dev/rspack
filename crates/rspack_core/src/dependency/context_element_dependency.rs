use itertools::Itertools;
use rspack_paths::Utf8Path;
use rspack_util::json_stringify;
use swc_core::ecma::atoms::Atom;

use super::AffectType;
use crate::{
  create_exports_object_referenced, AsContextDependency, AsDependencyTemplate, Context,
  ImportAttributes, ModuleLayer,
};
use crate::{ContextMode, ContextOptions, Dependency};
use crate::{DependencyCategory, DependencyId, DependencyType};
use crate::{ExtendedReferencedExport, ModuleDependency};
use crate::{ModuleGraph, ReferencedExport, RuntimeSpec};

#[derive(Debug, Clone)]
pub struct ContextElementDependency {
  pub id: DependencyId,
  // TODO remove this async dependency mark
  pub options: ContextOptions,
  pub request: String,
  pub user_request: String,
  pub category: DependencyCategory,
  pub context: Context,
  pub layer: Option<ModuleLayer>,
  pub resource_identifier: String,
  pub referenced_exports: Option<Vec<Atom>>,
  pub dependency_type: DependencyType,
  pub attributes: Option<ImportAttributes>,
}

impl ContextElementDependency {
  pub fn create_resource_identifier(
    resource: &str,
    path: &Utf8Path,
    attributes: Option<&ImportAttributes>,
  ) -> String {
    let mut ident = format!("context{resource}|{path}");
    if let Some(attributes) = attributes {
      ident += &json_stringify(&attributes);
    }
    ident
  }
}

impl Dependency for ContextElementDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    &self.dependency_type
  }

  fn get_context(&self) -> Option<&Context> {
    Some(&self.context)
  }

  fn get_layer(&self) -> Option<&ModuleLayer> {
    self.layer.as_ref()
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    self.attributes.as_ref()
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    if let Some(referenced_exports) = &self.referenced_exports {
      referenced_exports
        .iter()
        .map(|export| {
          ExtendedReferencedExport::Export(ReferencedExport::new(vec![export.clone()], false))
        })
        .collect_vec()
    } else {
      create_exports_object_referenced()
    }
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::True
  }
}

impl ModuleDependency for ContextElementDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.user_request
  }

  fn weak(&self) -> bool {
    matches!(
      self.options.mode,
      ContextMode::AsyncWeak | ContextMode::Weak
    )
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}

impl AsDependencyTemplate for ContextElementDependency {}
impl AsContextDependency for ContextElementDependency {}
