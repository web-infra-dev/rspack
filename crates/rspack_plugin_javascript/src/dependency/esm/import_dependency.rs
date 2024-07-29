use rspack_core::{
  create_exports_object_referenced, module_namespace_promise, DependencyType, ErrorSpan,
  ExportsType, ExtendedReferencedExport, ImportAttributes, ModuleGraph, ReferencedExport,
};
use rspack_core::{AsContextDependency, Dependency};
use rspack_core::{DependencyCategory, DependencyId, DependencyTemplate};
use rspack_core::{ModuleDependency, TemplateContext, TemplateReplaceSource};
use swc_core::ecma::atoms::Atom;

use super::create_resource_identifier_for_esm_dependency;

pub fn create_import_dependency_referenced_exports(
  dependency_id: &DependencyId,
  referenced_exports: &Option<Vec<Atom>>,
  mg: &ModuleGraph,
) -> Vec<ExtendedReferencedExport> {
  if let Some(referenced_exports) = referenced_exports {
    let mut refs = vec![];
    for referenced_export in referenced_exports {
      if referenced_export == "default" {
        let Some(strict) = mg
          .get_parent_module(dependency_id)
          .and_then(|id| mg.module_by_identifier(id))
          .and_then(|m| m.build_meta())
          .map(|bm| bm.strict_harmony_module)
        else {
          return create_exports_object_referenced();
        };
        let Some(imported_module) = mg
          .module_identifier_by_dependency_id(dependency_id)
          .and_then(|id| mg.module_by_identifier(id))
        else {
          return create_exports_object_referenced();
        };
        let exports_type = imported_module.get_exports_type_readonly(mg, strict);
        if matches!(
          exports_type,
          ExportsType::DefaultOnly | ExportsType::DefaultWithNamed
        ) {
          return create_exports_object_referenced();
        }
      }
      refs.push(ExtendedReferencedExport::Export(ReferencedExport::new(
        vec![referenced_export.clone()],
        false,
      )));
    }
    refs
  } else {
    create_exports_object_referenced()
  }
}

#[derive(Debug, Clone)]
pub struct ImportDependency {
  start: u32,
  end: u32,
  id: DependencyId,
  request: Atom,
  span: Option<ErrorSpan>,
  referenced_exports: Option<Vec<Atom>>,
  attributes: Option<ImportAttributes>,
  resource_identifier: String,
}

impl ImportDependency {
  pub fn new(
    start: u32,
    end: u32,
    request: Atom,
    span: Option<ErrorSpan>,
    referenced_exports: Option<Vec<Atom>>,
    attributes: Option<ImportAttributes>,
  ) -> Self {
    let resource_identifier =
      create_resource_identifier_for_esm_dependency(request.as_str(), attributes.as_ref());
    Self {
      start,
      end,
      request,
      span,
      id: DependencyId::new(),
      referenced_exports,
      attributes,
      resource_identifier,
    }
  }
}

impl Dependency for ImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::DynamicImport
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    self.attributes.as_ref()
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }

  fn get_referenced_exports(
    &self,
    module_graph: &rspack_core::ModuleGraph,
    _runtime: Option<&rspack_core::RuntimeSpec>,
  ) -> Vec<rspack_core::ExtendedReferencedExport> {
    create_import_dependency_referenced_exports(&self.id, &self.referenced_exports, module_graph)
  }
}

impl ModuleDependency for ImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }
}

impl DependencyTemplate for ImportDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let module_graph = code_generatable_context.compilation.get_module_graph();
    let block = module_graph.get_parent_block(&self.id);
    source.replace(
      self.start,
      self.end,
      module_namespace_promise(
        code_generatable_context,
        &self.id,
        block,
        &self.request,
        self.dependency_type().as_str(),
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

impl AsContextDependency for ImportDependency {}
