use rspack_core::ModuleDependency;
use swc_core::ecma::{
  ast::NewExpr,
  visit::{noop_visit_type, Visit, VisitWith},
};

use crate::dependency::URLDependency;

pub struct UrlScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  worker_syntax_list: &'a rspack_core::needs_refactor::WorkerSyntaxList,
}

// new URL("./foo.png", import.meta.url);
impl<'a> UrlScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    worker_syntax_list: &'a rspack_core::needs_refactor::WorkerSyntaxList,
  ) -> Self {
    Self {
      dependencies,
      worker_syntax_list,
    }
  }
}

impl Visit for UrlScanner<'_> {
  noop_visit_type!();

  fn visit_new_expr(&mut self, new_expr: &NewExpr) {
    // TODO: https://github.com/web-infra-dev/rspack/discussions/3619
    if self.worker_syntax_list.match_new_worker(new_expr) && let Some(args) = &new_expr.args {
      for arg in args.iter().skip(1) {
        arg.visit_with(self);
      }
      return;
    }
    if let Some((start, end, request)) = rspack_core::needs_refactor::match_new_url(new_expr) {
      self.dependencies.push(Box::new(URLDependency::new(
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
