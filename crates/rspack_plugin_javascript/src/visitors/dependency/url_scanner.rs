use rspack_core::{ModuleDependency, SpanExt};
use swc_core::common::Spanned;
use swc_core::ecma::{
  ast::{Expr, ExprOrSpread, Ident, Lit, NewExpr},
  atoms::js_word,
  visit::{noop_visit_type, Visit, VisitWith},
};

use super::expr_matcher;
use crate::dependency::URLDependency;
pub struct UrlScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
}

// new URL("./foo.png", import.meta.url);
impl<'a> UrlScanner<'a> {
  pub fn new(dependencies: &'a mut Vec<Box<dyn ModuleDependency>>) -> Self {
    Self { dependencies }
  }
}

impl Visit for UrlScanner<'_> {
  noop_visit_type!();

  fn visit_new_expr(&mut self, new_expr: &NewExpr) {
    if let Expr::Ident(Ident {
      sym: js_word!("URL"),
      ..
    }) = &*new_expr.callee
    {
      if let Some(args) = &new_expr.args {
        if let (Some(first), Some(second)) = (args.first(), args.get(1)) {
          if let (
            ExprOrSpread {
              spread: None,
              expr: box Expr::Lit(Lit::Str(path)),
            },
            ExprOrSpread {
              spread: None,
              expr:
                box expr
            },
          ) = (first, second) && expr_matcher::is_import_meta_url(expr)
          {
            self.dependencies.push(Box::new(URLDependency::new(
              path.span.real_lo(),
              expr.span().real_hi(),
              path.value.clone(),
              Some(new_expr.span.into()),
            )));
          }
        }
      }
    } else {
      new_expr.visit_children_with(self);
    }
  }
}
