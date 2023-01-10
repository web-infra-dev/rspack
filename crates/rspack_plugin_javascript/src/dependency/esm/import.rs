use rspack_core::{
  create_javascript_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  Dependency, DependencyCategory, DependencyType, ErrorSpan, JsAstPath, ModuleDependency,
  ModuleDependencyExt, ModuleIdentifier,
};
use swc_core::ecma::atoms::{Atom, JsWord};

#[derive(Debug, Eq, Clone)]
pub struct EsmImportDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: JsWord,
  // user_request: String,
  category: &'static DependencyCategory,
  dependency_type: &'static DependencyType,

  span: Option<ErrorSpan>,
  #[allow(unused)]
  ast_path: JsAstPath,
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl PartialEq for EsmImportDependency {
  fn eq(&self, other: &Self) -> bool {
    self.parent_module_identifier == other.parent_module_identifier
      && self.request == other.request
      && self.category == other.category
      && self.dependency_type == other.dependency_type
  }
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl std::hash::Hash for EsmImportDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.parent_module_identifier.hash(state);
    self.request.hash(state);
    self.category.hash(state);
    self.dependency_type.hash(state);
  }
}

impl EsmImportDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      request,
      // user_request,
      category: &DependencyCategory::Esm,
      dependency_type: &DependencyType::EsmImport,
      span,
      ast_path,
    }
  }
}

impl Dependency for EsmImportDependency {
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

impl ModuleDependency for EsmImportDependency {
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

impl CodeGeneratable for EsmImportDependency {
  fn generate(
    &self,
    code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    let CodeGeneratableContext { compilation, .. } = code_generatable_context;
    let mut code_gen = CodeGeneratableResult::default();

    let module_id = self
      .referencing_module_graph_module(&compilation.module_graph)
      .map(|m| m.id(&compilation.chunk_graph).to_string());

    if let Some(module_id) = module_id {
      code_gen.visitors.push(
        create_javascript_visitor!(exact &self.ast_path, visit_mut_module_decl(n: &mut ModuleDecl) {
          if let Some(import) = n.as_mut_import() {
            *import.src = Atom::from(&*module_id).into();
          }
        }),
      );
    }

    Ok(code_gen)
  }
}
