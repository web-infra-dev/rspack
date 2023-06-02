use rspack_core::{
  create_javascript_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableDeclMappings,
  CodeGeneratableResult, Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan,
  JsAstPath, ModuleDependency, ModuleDependencyExt,
};
use swc_core::ecma::atoms::{Atom, JsWord};

#[derive(Debug, Clone)]
pub struct EsmImportDependency {
  id: Option<DependencyId>,
  request: JsWord,
  // user_request: String,
  category: &'static DependencyCategory,
  dependency_type: &'static DependencyType,

  span: Option<ErrorSpan>,
  #[allow(unused)]
  ast_path: JsAstPath,
}

impl EsmImportDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      request,
      // user_request,
      category: &DependencyCategory::Esm,
      dependency_type: &DependencyType::EsmImport,
      span,
      ast_path,
      id: None,
    }
  }
}

impl Dependency for EsmImportDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
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

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }
}

impl CodeGeneratable for EsmImportDependency {
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
          if let Some(import) = n.as_mut_import() {
            *import.src = Atom::from(&*module_id).into();
          }
        }),
      );
      }
    }

    Ok(code_gen.with_decl_mappings(decl_mappings))
  }
}
