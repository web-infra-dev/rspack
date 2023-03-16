use rspack_core::{
  CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
  DependencyId, DependencyType, ErrorSpan, JsAstPath, ModuleDependency, ModuleIdentifier,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Eq, Clone)]
pub struct ModuleHotAcceptDependency {
  id: Option<DependencyId>,
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
impl PartialEq for ModuleHotAcceptDependency {
  fn eq(&self, other: &Self) -> bool {
    self.parent_module_identifier == other.parent_module_identifier
      && self.request == other.request
      && self.category == other.category
      && self.dependency_type == other.dependency_type
  }
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl std::hash::Hash for ModuleHotAcceptDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.parent_module_identifier.hash(state);
    self.request.hash(state);
    self.category.hash(state);
    self.dependency_type.hash(state);
  }
}

impl ModuleHotAcceptDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      id: None,
      parent_module_identifier: None,
      request,
      category: &DependencyCategory::CommonJS,
      dependency_type: &DependencyType::ModuleHotAccept,
      span,
      ast_path,
    }
  }
}

impl Dependency for ModuleHotAcceptDependency {
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

impl ModuleDependency for ModuleHotAcceptDependency {
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

impl CodeGeneratable for ModuleHotAcceptDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    // let CodeGeneratableContext { compilation, .. } = code_generatable_context;

    let code_gen = CodeGeneratableResult::default();

    // if let Some(module_id) = compilation
    //   .module_graph
    //   .module_graph_module_by_dependency_id(self.id().expect("should have id"))
    //   .map(|m| m.id(&compilation.chunk_graph).to_string())
    // {
    //   code_gen.visitors.push(
    //     create_javascript_visitor!(exact &self.ast_path, visit_mut_call_expr(n: &mut CallExpr) {
    //       if let Some(Lit::Str(str)) = n
    //         .args
    //         .get_mut(0)
    //         .and_then(|first_arg| first_arg.expr.as_mut_lit())
    //       {
    //         str.value = JsWord::from(&*module_id);
    //         str.raw = Some(Atom::from(format!("\"{module_id}\"")));
    //       }

    //     }),
    //   );
    // }

    Ok(code_gen)
  }
}
