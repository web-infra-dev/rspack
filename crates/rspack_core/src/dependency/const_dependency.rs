use rspack_error::Result;
use swc_core::ecma::{
  ast::Expr,
  visit::{fields::ExprField, AstParentKind},
};

use crate::{
  create_javascript_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  Dependency, JsAstPath, ModuleIdentifier,
};

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ConstDependency {
  pub expression: Expr,
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

    let expr = self.expression.clone();

    let mut is_member = false;
    if let Some(path) = self.ast_path.last() {
      if let AstParentKind::Expr(expr_field) = path {
        if let ExprField::Member = expr_field {
          is_member = true;
          let member = match expr.clone() {
            Expr::Member(member) => member,
            _ => unreachable!(),
          };
          cgr.visitors.push(
            create_javascript_visitor!(exact &self.ast_path, visit_mut_member_expr(n: &mut MemberExpr) {
              *n = member.clone();
            }),
          );
        }
      }
    }
    if !is_member {
      cgr.visitors.push(
        create_javascript_visitor!(exact &self.ast_path, visit_mut_expr(n: &mut Expr) {
          *n = expr.clone();
        }),
      );
    }

    Ok(cgr)
  }
}

impl ConstDependency {
  pub fn new(
    expression: Expr,
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
