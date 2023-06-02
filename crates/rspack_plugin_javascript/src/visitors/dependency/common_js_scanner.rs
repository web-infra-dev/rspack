use rspack_core::{
  ConstDependency, Dependency, ModuleDependency, RequireResolveDependency, RuntimeGlobals,
  RuntimeRequirementsDependency,
};
use swc_core::common::pass::AstNodePath;
use swc_core::ecma::ast::{CallExpr, Expr, Lit};
use swc_core::ecma::visit::fields::IfStmtField;
use swc_core::ecma::visit::{AstParentKind, AstParentNodeRef, VisitAstPath, VisitWithPath};
use swc_core::quote;

use super::{as_parent_path, expr_matcher, is_require_resolve_call, is_require_resolve_weak_call};

pub struct CommonJsScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
  in_try: bool,
}

impl<'a> CommonJsScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
  ) -> Self {
    Self {
      dependencies,
      presentational_dependencies,
      in_try: false,
    }
  }

  fn add_dependency(&mut self, dependency: Box<dyn ModuleDependency>) {
    self.dependencies.push(dependency);
  }

  fn add_presentational_dependency(&mut self, dependency: Box<dyn Dependency>) {
    self.presentational_dependencies.push(dependency);
  }

  fn add_require_resolve(
    &mut self,
    node: &CallExpr,
    ast_path: &AstNodePath<AstParentNodeRef<'_>>,
    weak: bool,
  ) {
    let mut ast_path = as_parent_path(ast_path);
    // add_require_resolve at visit_call_expr, but we want use visit_expr at generate
    ast_path.pop();
    if !node.args.is_empty() {
      if let Some(Lit::Str(str)) = node.args.get(0).and_then(|x| x.expr.as_lit()) {
        self.add_dependency(Box::new(RequireResolveDependency::new(
          str.value.to_string(),
          weak,
          node.span.into(),
          ast_path,
          self.in_try,
        )));
      }
    }
  }
}

impl VisitAstPath for CommonJsScanner<'_> {
  fn visit_try_stmt<'ast: 'r, 'r>(
    &mut self,
    node: &'ast swc_core::ecma::ast::TryStmt,
    ast_path: &mut swc_core::ecma::visit::AstNodePath<'r>,
  ) {
    self.in_try = true;
    node.visit_children_with_path(self, ast_path);
    self.in_try = false;
  }

  fn visit_expr<'ast: 'r, 'r>(
    &mut self,
    expr: &'ast Expr,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    if expr_matcher::is_module_id(expr) {
      self.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        RuntimeGlobals::MODULE_ID,
      )));
    }
    if expr_matcher::is_module_loaded(expr) {
      self.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        RuntimeGlobals::MODULE_LOADED,
      )));
    }
    // only evaluate require.resolveWeak and require.resolve to true in IfStmt::Test
    if (expr_matcher::is_require_resolve_weak(expr) || expr_matcher::is_require_resolve(expr))
      && ast_path
        .iter()
        .rev()
        .any(|i| i.kind() == AstParentKind::IfStmt(IfStmtField::Test))
    {
      self.add_presentational_dependency(Box::new(ConstDependency::new(
        quote!("true" as Expr),
        None,
        as_parent_path(ast_path),
      )));
    }
    expr.visit_children_with_path(self, ast_path);
  }

  fn visit_call_expr<'ast: 'r, 'r>(
    &mut self,
    node: &'ast CallExpr,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    if is_require_resolve_call(node) {
      return self.add_require_resolve(node, ast_path, false);
    }
    if is_require_resolve_weak_call(node) {
      return self.add_require_resolve(node, ast_path, true);
    }
    node.visit_children_with_path(self, ast_path);
  }
}
