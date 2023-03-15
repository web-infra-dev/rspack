use rspack_core::{
  create_javascript_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan, JsAstPath,
  ModuleDependency, ModuleIdentifier,
};
use rspack_core::{CodeGeneratableDeclMappings, ModuleDependencyExt};
use swc_core::ecma::ast::ModuleDecl;
use swc_core::ecma::atoms::Atom;
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Eq, Clone)]
pub struct EsmExportDependency {
  id: Option<DependencyId>,
  parent_module_identifier: Option<ModuleIdentifier>,
  request: JsWord,
  category: &'static DependencyCategory,
  dependency_type: &'static DependencyType,

  span: Option<ErrorSpan>,
  #[allow(unused)]
  ast_path: JsAstPath,
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl PartialEq for EsmExportDependency {
  fn eq(&self, other: &Self) -> bool {
    self.parent_module_identifier == other.parent_module_identifier
      && self.request == other.request
      && self.category == other.category
      && self.dependency_type == other.dependency_type
  }
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl std::hash::Hash for EsmExportDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.parent_module_identifier.hash(state);
    self.request.hash(state);
    self.category.hash(state);
    self.dependency_type.hash(state);
  }
}

impl EsmExportDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      request,
      category: &DependencyCategory::Esm,
      dependency_type: &DependencyType::EsmExport,
      span,
      ast_path,
      id: None,
    }
  }
}

impl Dependency for EsmExportDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = module_identifier;
  }

  fn category(&self) -> &DependencyCategory {
    self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    self.dependency_type
  }
}

impl ModuleDependency for EsmExportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }
}

impl CodeGeneratable for EsmExportDependency {
  fn generate(
    &self,
    code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    let CodeGeneratableContext { compilation, .. } = code_generatable_context;

    let mut code_gen = CodeGeneratableResult::default();
    let mut decl_mappings = CodeGeneratableDeclMappings::default();

    if let Some(id) = self.id() {
      if let Some(module_id) = compilation
        .module_graph
        .module_graph_module_by_dependency_id(&id)
        .map(|m| m.id(&compilation.chunk_graph).to_string())
      {
        {
          let (id, val) = self.decl_mapping(&compilation.module_graph, module_id.clone());
          decl_mappings.insert(id, val);
        }

        code_gen.visitors.push(
        create_javascript_visitor!(exact &self.ast_path, visit_mut_module_decl(n: &mut ModuleDecl) {
          match n {
            ModuleDecl::ExportAll(e) => {
              e.src.value = JsWord::from(&*module_id);
              e.src.raw = Some(Atom::from(format!("\"{module_id}\"")));
            }
            ModuleDecl::ExportNamed(e) => {
              if let Some(e) = e.src.as_mut() {
                e.value = JsWord::from(&*module_id);
                e.raw = Some(Atom::from(format!("\"{module_id}\"")));
              }
            }
            _ => {}
          }
        }),
      );
      }
    }

    Ok(code_gen.with_decl_mappings(decl_mappings))
  }
}
