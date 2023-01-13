use rspack_core::{
  create_javascript_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  Dependency, DependencyCategory, DependencyType, ErrorSpan, JsAstPath, ModuleDependency,
  ModuleDependencyExt, ModuleIdentifier,
};
use swc_core::ecma::{
  ast::*,
  atoms::{Atom, JsWord},
};

#[derive(Debug, Eq, Clone)]
pub struct CommonJSRequireDependency {
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
impl PartialEq for CommonJSRequireDependency {
  fn eq(&self, other: &Self) -> bool {
    self.parent_module_identifier == other.parent_module_identifier
      && self.request == other.request
      && self.category == other.category
      && self.dependency_type == other.dependency_type
  }
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl std::hash::Hash for CommonJSRequireDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.parent_module_identifier.hash(state);
    self.request.hash(state);
    self.category.hash(state);
    self.dependency_type.hash(state);
  }
}

impl CommonJSRequireDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      request,
      // user_request,
      category: &DependencyCategory::CommonJS,
      dependency_type: &DependencyType::CjsRequire,
      span,
      ast_path,
    }
  }
}

impl Dependency for CommonJSRequireDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = module_identifier;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsRequire
  }
}

impl ModuleDependency for CommonJSRequireDependency {
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

impl CodeGeneratable for CommonJSRequireDependency {
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
        create_javascript_visitor!(exact &self.ast_path, visit_mut_call_expr(n: &mut CallExpr) {
          if let Callee::Expr(box Expr::Ident(_ident)) = &mut n.callee {
            if let Some(ExprOrSpread {
              spread: None,
              expr: box Expr::Lit(Lit::Str(str)),
            }) = n.args.first_mut()
            {
              str.value = JsWord::from(&*module_id);
              str.raw = Some(Atom::from(format!("\"{module_id}\"")));
            };
          }
        }),
      );
    }

    Ok(code_gen)
  }
}
