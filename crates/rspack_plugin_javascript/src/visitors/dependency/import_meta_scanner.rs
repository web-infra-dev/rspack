use rspack_core::{
  CodeGeneratableDependency, CompilerOptions, ConstDependency, ResourceData, SpanExt,
};
use swc_core::common::Spanned;
use swc_core::ecma::ast::{Expr, Ident, NewExpr, UnaryExpr, UnaryOp};
use swc_core::ecma::atoms::js_word;
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};
use url::Url;

use super::{
  expr_matcher, is_member_expr_starts_with_import_meta,
  is_member_expr_starts_with_import_meta_webpack_hot,
};

// Port from https://github.com/webpack/webpack/blob/main/lib/dependencies/ImportMetaPlugin.js
// TODO:
// - scan `import.meta.webpack`
// - scan `import.meta.url.indexOf("index.js")`
// - evaluate expression. eg `import.meta.env && import.meta.env.xx` should be `false`
// - add warning for `import.meta`
pub struct ImportMetaScanner<'a> {
  pub presentational_dependencies: &'a mut Vec<Box<dyn CodeGeneratableDependency>>,
  pub compiler_options: &'a CompilerOptions,
  pub resource_data: &'a ResourceData,
}

impl<'a> ImportMetaScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn CodeGeneratableDependency>>,
    resource_data: &'a ResourceData,
    compiler_options: &'a CompilerOptions,
  ) -> Self {
    Self {
      presentational_dependencies,
      resource_data,
      compiler_options,
    }
  }
}

impl Visit for ImportMetaScanner<'_> {
  noop_visit_type!();

  fn visit_unary_expr(&mut self, unary_expr: &UnaryExpr) {
    if let UnaryExpr {
      op: UnaryOp::TypeOf,
      arg: box expr,
      ..
    } = unary_expr
    {
      if expr_matcher::is_import_meta(expr) {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            unary_expr.span().real_lo(),
            unary_expr.span().real_hi(),
            "'object'".into(),
            None,
          )));
      } else if expr_matcher::is_import_meta_url(expr) {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            unary_expr.span().real_lo(),
            unary_expr.span().real_hi(),
            "'string'".into(),
            None,
          )));
      } else if is_member_expr_starts_with_import_meta(expr) {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            unary_expr.span().real_lo(),
            unary_expr.span().real_hi(),
            "'undefined'".into(),
            None,
          )));
      }
    } else {
      unary_expr.visit_children_with(self);
    }
  }

  fn visit_expr(&mut self, expr: &Expr) {
    // exclude import.meta.webpackHot
    if is_member_expr_starts_with_import_meta_webpack_hot(expr) {
      return;
    }

    // import.meta
    if expr_matcher::is_import_meta(expr) {
      // TODO(underfin): add warning
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          "({})".into(),
          None,
        )));
    } else if expr_matcher::is_import_meta_url(expr) {
      // import.meta.url
      let url = Url::from_file_path(&self.resource_data.resource).expect("should be a path");
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          format!("'{url}'").into(),
          None,
        )));
    } else if is_member_expr_starts_with_import_meta(expr) {
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          "undefined".into(),
          None,
        )));
    } else {
      expr.visit_children_with(self);
    }
  }

  fn visit_new_expr(&mut self, new_expr: &NewExpr) {
    // exclude new URL("", import.meta.url)
    if let Expr::Ident(Ident {
      sym: js_word!("URL"),
      ..
    }) = &*new_expr.callee
    {
      return;
    }
    new_expr.visit_children_with(self);
  }
}
