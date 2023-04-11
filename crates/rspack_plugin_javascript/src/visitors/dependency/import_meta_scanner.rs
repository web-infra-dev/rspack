use rspack_core::{CompilerOptions, ConstDependency, Dependency, ResourceData};
use swc_core::common::pass::AstNodePath;
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::{Expr, Lit, Str, UnaryExpr, UnaryOp};
use swc_core::ecma::visit::{AstParentNodeRef, VisitAstPath, VisitWithPath};
use swc_core::quote;
use url::Url;

use super::{
  as_parent_path, is_import_meta, is_import_meta_hot, is_import_meta_member_expr,
  match_import_meta_member_expr,
};

// Port from https://github.com/webpack/webpack/blob/main/lib/dependencies/ImportMetaPlugin.js
// TODO:
// - scan `import.meta.webpack`
// - scan `import.meta.url.indexOf("index.js")`
// - evaluate expression. eg `import.meta.env && import.meta.env.xx` should be `false`
// - add warning for `import.meta`
pub struct ImportMetaScanner<'a> {
  pub presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
  pub compiler_options: &'a CompilerOptions,
  pub resource_data: &'a ResourceData,
}

impl<'a> ImportMetaScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
    resource_data: &'a ResourceData,
    compiler_options: &'a CompilerOptions,
  ) -> Self {
    Self {
      presentational_dependencies,
      resource_data,
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
    // exclude import.meta.webpackHot
    if is_import_meta_hot(expr) {
      return;
    }
    // import.meta.url
    if match_import_meta_member_expr(expr, "import.meta.url") {
      let url = Url::from_file_path(&self.resource_data.resource).expect("should be a path");
      self.add_presentational_dependency(box ConstDependency::new(
        Expr::Lit(Lit::Str(Str {
          span: DUMMY_SP,
          value: format!("'{}'", url.as_str()).into(),
          raw: Some(format!("'{}'", url.as_str()).into()),
        })),
        None,
        as_parent_path(ast_path),
      ));
    }
    // import.meta.xxx
    else if is_import_meta_member_expr(expr) {
      self.add_presentational_dependency(box ConstDependency::new(
        quote!("undefined" as Expr),
        None,
        as_parent_path(ast_path),
      ));
    }
    // import.meta
    else if is_import_meta(expr) {
      // TODO add warning
      self.add_presentational_dependency(box ConstDependency::new(
        quote!("({})" as Expr),
        None,
        as_parent_path(ast_path),
      ));
    } else if let Expr::Unary(UnaryExpr {
      op: UnaryOp::TypeOf,
      arg: box expr,
      ..
    }) = expr
    {
      // typeof import.meta.url
      if match_import_meta_member_expr(expr, "import.meta.url") {
        self.add_presentational_dependency(box ConstDependency::new(
          quote!("'string'" as Expr),
          None,
          as_parent_path(ast_path),
        ));
      }
      // typeof import.meta.xxx
      else if is_import_meta_member_expr(expr) {
        self.add_presentational_dependency(box ConstDependency::new(
          quote!("undefined" as Expr),
          None,
          as_parent_path(ast_path),
        ));
      }
      // typeof import.meta
      else if is_import_meta(expr) {
        self.add_presentational_dependency(box ConstDependency::new(
          quote!("'object'" as Expr),
          None,
          as_parent_path(ast_path),
        ));
      }
    }

    expr.visit_children_with_path(self, ast_path);
  }
}
