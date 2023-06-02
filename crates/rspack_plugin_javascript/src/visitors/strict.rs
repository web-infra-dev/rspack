use rspack_core::BuildInfo;
use swc_core::ecma::ast::{Expr, ExprStmt, Lit, Stmt, Str};
use swc_core::ecma::visit::{as_folder, noop_visit_mut_type, Fold, VisitMut};

pub fn strict_mode(build_info: &mut BuildInfo) -> impl Fold + '_ {
  as_folder(StrictModeVisitor { build_info })
}

struct StrictModeVisitor<'a> {
  build_info: &'a mut BuildInfo,
}

impl<'a> VisitMut for StrictModeVisitor<'a> {
  noop_visit_mut_type!();

  fn visit_mut_stmt(&mut self, stmt: &mut Stmt) {
    if self.build_info.strict {
      return;
    }
    if let Stmt::Expr(ExprStmt { box expr, .. }) = stmt && let Expr::Lit(Lit::Str(Str { ref value, .. })) = expr && value == "use strict" {
       self.build_info.strict = true;
      }
  }
}
