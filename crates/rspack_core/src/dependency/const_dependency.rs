use rspack_error::Result;
use swc_core::ecma::ast::Expr;

use crate::{
  create_javascript_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  Dependency, JsAstPath, RuntimeGlobals,
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ConstDependency {
  pub expression: Expr,
  pub runtime_requirements: Option<RuntimeGlobals>,
  #[allow(unused)]
  pub ast_path: JsAstPath,
}

impl Dependency for ConstDependency {}

impl CodeGeneratable for ConstDependency {
  fn generate(
    &self,
    code_generatable_context: &mut CodeGeneratableContext,
  ) -> Result<CodeGeneratableResult> {
    let mut cgr = CodeGeneratableResult::default();

    if let Some(runtime_requirement) = self.runtime_requirements {
      code_generatable_context
        .runtime_requirements
        .insert(runtime_requirement);
    }

    let expr = self.expression.clone();
    cgr.visitors.push(
      create_javascript_visitor!(exact &self.ast_path, visit_mut_expr(n: &mut Expr) {
        *n = expr.clone();
      }),
    );

    Ok(cgr)
  }
}

impl ConstDependency {
  pub fn new(
    expression: Expr,
    runtime_requirements: Option<RuntimeGlobals>,
    ast_path: JsAstPath,
  ) -> Self {
    Self {
      expression,
      runtime_requirements,
      ast_path,
    }
  }
}
