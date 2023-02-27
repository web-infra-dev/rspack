use rspack_error::Result;
use swc_core::ecma::{ast::Expr, utils::quote_ident};

use crate::{
  create_javascript_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  Dependency, JsAstPath, ModuleIdentifier,
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ConstDependency {
  pub expression: String,
  pub runtime_requirements: Option<&'static str>,
  #[allow(unused)]
  pub ast_path: JsAstPath,
}

impl Dependency for ConstDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    None
  }
}

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

    let str = self.expression.to_string();
    cgr.visitors.push(
      create_javascript_visitor!(exact &self.ast_path, visit_mut_expr(n: &mut Expr) {
        *n = Expr::Ident(quote_ident!(*str));
      }),
    );

    Ok(cgr)
  }
}

impl ConstDependency {
  pub fn new(
    expression: String,
    runtime_requirements: Option<&'static str>,
    ast_path: JsAstPath,
  ) -> Self {
    Self {
      expression,
      runtime_requirements,
      ast_path,
    }
  }
}
