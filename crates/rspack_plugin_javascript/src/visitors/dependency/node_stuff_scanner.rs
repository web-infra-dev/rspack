use rspack_core::{
  CompilerOptions, ConstDependency, Dependency, NodeOption, ResourceData, RuntimeGlobals,
};
use sugar_path::SugarPath;
use swc_core::common::pass::AstNodePath;
use swc_core::common::SyntaxContext;
use swc_core::ecma::ast::{Expr, Lit};
use swc_core::ecma::utils::{quote_ident, quote_str};
use swc_core::ecma::visit::{AstParentNodeRef, VisitAstPath, VisitWithPath};

use super::as_parent_path;

const DIR_NAME: &str = "__dirname";
const FILE_NAME: &str = "__filename";
const GLOBAL: &str = "global";

pub struct NodeStuffScanner<'a> {
  pub presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
  pub unresolved_ctxt: &'a SyntaxContext,
  pub compiler_options: &'a CompilerOptions,
  pub node_option: &'a NodeOption,
  pub resource_data: &'a ResourceData,
}

impl<'a> NodeStuffScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
    unresolved_ctxt: &'a SyntaxContext,
    compiler_options: &'a CompilerOptions,
    node_option: &'a NodeOption,
    resource_data: &'a ResourceData,
  ) -> Self {
    Self {
      presentational_dependencies,
      unresolved_ctxt,
      compiler_options,
      node_option,
      resource_data,
    }
  }

  fn add_presentational_dependency(&mut self, dependency: Box<dyn Dependency>) {
    self.presentational_dependencies.push(dependency);
  }
}

impl VisitAstPath for NodeStuffScanner<'_> {
  fn visit_expr<'ast: 'r, 'r>(
    &mut self,
    expr: &'ast Expr,
    ast_path: &mut AstNodePath<AstParentNodeRef<'r>>,
  ) {
    if let Expr::Ident(ident) = expr {
      if ident.span.ctxt == *self.unresolved_ctxt {
        match ident.sym.as_ref() as &str {
          DIR_NAME => {
            let dirname = match self.node_option.dirname.as_str() {
              "mock" => Some("/".to_string()),
              "warn-mock" => Some("/".to_string()),
              "true" => Some(
                self
                  .resource_data
                  .resource_path
                  .parent()
                  .expect("TODO:")
                  .relative(self.compiler_options.context.as_ref())
                  .to_string_lossy()
                  .to_string(),
              ),
              _ => None,
            };
            if let Some(dirname) = dirname {
              self.add_presentational_dependency(Box::new(ConstDependency::new(
                Expr::Lit(Lit::Str(quote_str!(dirname))),
                None,
                as_parent_path(ast_path),
              )));
            }
          }
          FILE_NAME => {
            let filename = match self.node_option.filename.as_str() {
              "mock" => Some("/index.js".to_string()),
              "warn-mock" => Some("/index.js".to_string()),
              "true" => Some(
                self
                  .resource_data
                  .resource_path
                  .relative(self.compiler_options.context.as_ref())
                  .to_string_lossy()
                  .to_string(),
              ),
              _ => None,
            };
            if let Some(filename) = filename {
              self.add_presentational_dependency(Box::new(ConstDependency::new(
                Expr::Lit(Lit::Str(quote_str!(filename))),
                None,
                as_parent_path(ast_path),
              )));
            }
          }
          GLOBAL => {
            if matches!(self.node_option.global.as_str(), "true" | "warn") {
              self.add_presentational_dependency(Box::new(ConstDependency::new(
                Expr::Ident(quote_ident!(RuntimeGlobals::GLOBAL)),
                Some(RuntimeGlobals::GLOBAL),
                as_parent_path(ast_path),
              )));
            }
          }
          _ => {}
        }
      }
    }
    expr.visit_children_with_path(self, ast_path);
  }
}
