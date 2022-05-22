use rspack_core::{
  ast::{Expr, ExprStmt, ModuleItem, Pat, PatOrExpr, Stmt},
  get_swc_compiler,
};
use rspack_swc::{
  swc_common::FileName,
  swc_ecma_parser::{Parser, StringInput, Syntax},
  swc_ecma_utils::DropSpan,
  swc_ecma_visit::{as_folder, FoldWith, VisitMut, VisitMutWith},
  swc_plugin::utils::take::Take,
};

struct Normalizer;

impl VisitMut for Normalizer {
  fn visit_mut_pat_or_expr(&mut self, node: &mut PatOrExpr) {
    node.visit_mut_children_with(self);

    if let PatOrExpr::Pat(pat) = node {
      if let Pat::Expr(e) = &mut **pat {
        *node = PatOrExpr::Expr(e.take());
      }
    }
  }
}

pub fn parse_expr(src: &str) -> Expr {
  let compiler = get_swc_compiler();
  let fm = compiler
    .cm
    .new_source_file(FileName::Real("ast_optimization.js".into()), src.into());

  let module = {
    let mut p = Parser::new(Syntax::default(), StringInput::from(&*fm), None);
    p.parse_module().expect("parse module failed")
  };

  let mut module = module
    .fold_with(&mut as_folder(DropSpan {
      preserve_ctxt: true,
    }))
    .fold_with(&mut as_folder(Normalizer));

  assert_eq!(module.body.len(), 1);

  let v = match module.body.pop().unwrap() {
    ModuleItem::Stmt(Stmt::Expr(ExprStmt { expr, .. })) => *expr,
    _ => unreachable!(),
  };

  v
}
