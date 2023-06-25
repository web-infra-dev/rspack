use rspack_core::{ModuleDependency, SpanExt};
use swc_core::common::Spanned;
use swc_core::ecma::{
  ast::{Expr, ExprOrSpread, Ident, Lit, NewExpr},
  atoms::js_word,
  visit::{noop_visit_type, Visit, VisitWith},
};

use super::expr_matcher;
use super::worker_scanner::WorkerScanner;
use crate::dependency::URLDependency;

pub struct UrlScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  worker_scanner: &'a WorkerScanner<'a>,
}

// new URL("./foo.png", import.meta.url);
impl<'a> UrlScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    worker_scanner: &'a WorkerScanner<'a>,
  ) -> Self {
    Self {
      dependencies,
      worker_scanner,
    }
  }
}

pub fn match_new_url(new_expr: &NewExpr) -> Option<(u32, u32, String)> {
  if matches!(&*new_expr.callee, Expr::Ident(Ident { sym: js_word!("URL"), .. }))
  && let Some(args) = &new_expr.args
  && let (Some(first), Some(second)) = (args.first(), args.get(1))
  && let (
    ExprOrSpread { spread: None, expr: box Expr::Lit(Lit::Str(path)) },
    ExprOrSpread { spread: None, expr: box expr },
  ) = (first, second) && expr_matcher::is_import_meta_url(expr) {
    return Some((path.span.real_lo(), expr.span().real_hi(), path.value.to_string()))
  }
  None
}

impl Visit for UrlScanner<'_> {
  noop_visit_type!();

  fn visit_new_expr(&mut self, new_expr: &NewExpr) {
    // TODO: https://github.com/web-infra-dev/rspack/discussions/3619
    if self.worker_scanner.match_new_worker(new_expr) && let Some(args) = &new_expr.args {
      for arg in args.iter().skip(1) {
        arg.visit_with(self);
      }
      return;
    }
    if let Some((start, end, request)) = match_new_url(new_expr) {
      self.dependencies.push(Box::new(NewURLDependency::new(
        start,
        end,
        request.into(),
        Some(new_expr.span.into()),
      )));
    } else {
      new_expr.visit_children_with(self);
    }
  }
}
