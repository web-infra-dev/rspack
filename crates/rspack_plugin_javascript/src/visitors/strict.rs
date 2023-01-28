use rspack_ast::javascript::Context;
use rspack_core::BuildInfo;
use swc_core::ecma::ast::{Expr, ExprStmt, Lit, ModuleItem, Stmt, Str};
use swc_core::ecma::visit::{as_folder, noop_visit_mut_type, Fold, VisitMut};

pub fn strict_mode<'a>(build_info: &'a mut BuildInfo, context: &'a Context) -> impl Fold + 'a {
  if context.is_esm {
    build_info.strict = true;
  }
  as_folder(StrictModeVisitor { build_info })
}

struct StrictModeVisitor<'a> {
  build_info: &'a mut BuildInfo,
}

impl<'a> VisitMut for StrictModeVisitor<'a> {
  noop_visit_mut_type!();

  fn visit_mut_module_item(&mut self, module_item: &mut ModuleItem) {
    if let ModuleItem::Stmt(Stmt::Expr(ExprStmt { expr, .. })) = module_item {
      if let Expr::Lit(Lit::Str(Str { ref value, .. })) = **expr {
        if value == "use strict" {
          self.build_info.strict = true;
        }
      }
    }
  }
}
