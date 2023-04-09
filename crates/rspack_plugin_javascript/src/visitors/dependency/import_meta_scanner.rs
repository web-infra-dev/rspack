use rspack_core::{
  CompilerOptions, ConstDependency, Dependency, NodeOption, ResourceData, RuntimeGlobals,
};
use swc_core::common::pass::AstNodePath;
use swc_core::common::SyntaxContext;
use swc_core::ecma::ast::{Expr, Lit};
use swc_core::ecma::utils::{quote_ident, quote_str};
use swc_core::ecma::visit::{AstParentNodeRef, VisitAstPath, VisitWithPath};

use super::as_parent_path;

pub struct ImportMetaScanner<'a> {
  pub presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
  pub unresolved_ctxt: &'a SyntaxContext,
  pub compiler_options: &'a CompilerOptions,
}

impl<'a> ImportMetaScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
    unresolved_ctxt: &'a SyntaxContext,
    compiler_options: &'a CompilerOptions,
  ) -> Self {
    Self {
      presentational_dependencies,
      unresolved_ctxt,
      compiler_options,
    }
  }

  fn add_presentational_dependency(&mut self, dependency: Box<dyn Dependency>) {
    self.presentational_dependencies.push(dependency);
  }
}

impl VisitAstPath for ImportMetaScanner<'_> {
  fn visit_expr<'ast: 'r, 'r>(
    &mut self,
    expr: &'ast Expr,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    if let Expr::Ident(ident) = expr {
      if ident.span.ctxt == *self.unresolved_ctxt {}
    }
    expr.visit_children_with_path(self, ast_path);
  }
}
