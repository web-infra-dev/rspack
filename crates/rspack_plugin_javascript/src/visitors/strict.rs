use rspack_core::{BuildInfo, BuildMeta};
use swc_core::ecma::ast::{Expr, ExprStmt, Lit, ModuleItem, Stmt, Str};
use swc_core::ecma::visit::{as_folder, noop_visit_mut_type, Fold, VisitMut};

pub fn strict_mode<'a>(
  build_info: &'a mut BuildInfo,
  build_meta: &'a mut BuildMeta,
) -> impl Fold + 'a {
  as_folder(StrictModeVisitor {
    build_info,
    build_meta,
  })
}

struct StrictModeVisitor<'a> {
  build_info: &'a mut BuildInfo,
  build_meta: &'a mut BuildMeta,
}

impl<'a> VisitMut for StrictModeVisitor<'a> {
  noop_visit_mut_type!();

  fn visit_mut_module_item(&mut self, module_item: &mut ModuleItem) {
    if matches!(module_item, ModuleItem::ModuleDecl(_)) {
      self.build_info.strict = true;
      self.build_meta.esm = true;
    }
    if let ModuleItem::Stmt(Stmt::Expr(ExprStmt { expr, .. })) = module_item {
      if let Expr::Lit(Lit::Str(Str { ref value, .. })) = **expr {
        if value == "use strict" {
          self.build_info.strict = true;
        }
      }
    }
  }
}
