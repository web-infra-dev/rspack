use rspack_error::Result;
use swc_core::{
  common::DUMMY_SP,
  ecma::ast::{Expr, Lit, Null, Str},
};

use crate::{
  create_javascript_visitor, CodeGeneratable, CodeGeneratableResult, ContextOptions, Dependency,
  DependencyId, ErrorSpan, JsAstPath, ModuleDependency,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequireResolveDependency {
  pub id: Option<DependencyId>,
  pub request: String,
  pub weak: bool,
  span: ErrorSpan,
  #[allow(unused)]
  pub ast_path: JsAstPath,
  optional: bool,
}

impl RequireResolveDependency {
  pub fn new(
    request: String,
    weak: bool,
    span: ErrorSpan,
    ast_path: JsAstPath,
    optional: bool,
  ) -> Self {
    Self {
      request,
      weak,
      span,
      ast_path,
      id: None,
      optional,
    }
  }
}

impl Dependency for RequireResolveDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }
  fn category(&self) -> &crate::DependencyCategory {
    &crate::DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &crate::DependencyType {
    &crate::DependencyType::RequireResolve
  }
}

impl ModuleDependency for RequireResolveDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&crate::ErrorSpan> {
    Some(&self.span)
  }

  fn weak(&self) -> bool {
    self.weak
  }

  fn options(&self) -> Option<&ContextOptions> {
    None
  }

  fn get_optional(&self) -> bool {
    self.optional
  }
}

impl CodeGeneratable for RequireResolveDependency {
  fn generate(&self, context: &mut crate::CodeGeneratableContext) -> Result<CodeGeneratableResult> {
    let compilation = context.compilation;
    let mut code_gen = CodeGeneratableResult::default();
    if let Some(id) = self.id() {
      if let Some(module_identifier) = compilation.module_graph.module_identifier_by_dependency_id(&id)
        && let Some(module_id) = compilation.chunk_graph.get_module_id(*module_identifier) {
        let module_id = module_id.to_string();
        code_gen.visitors.push(
          create_javascript_visitor!(exact &self.ast_path, visit_mut_expr(n: &mut Expr) {
            *n = Expr::Lit(Lit::Str(Str { span: DUMMY_SP, value: module_id.clone().into(), raw: None }))
          }),
        );
      } else if self.weak() {
        code_gen.visitors.push(
          create_javascript_visitor!(exact &self.ast_path, visit_mut_expr(n: &mut Expr) {
            *n = Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))
          }),
        );
      }
    }
    Ok(code_gen)
  }
}
