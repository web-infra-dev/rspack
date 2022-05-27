use rspack_core::{
  ast::{Expr, ExprStmt, ModuleItem, Pat, PatOrExpr, Stmt},
  get_swc_compiler,
};
use rspack_swc::{
  swc_common::{FileName, Span},
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

pub fn replace_span_for_stmt(mut stmt: Stmt, span: Span) -> Stmt {
  match &mut stmt {
    Stmt::Block(s) => {
      s.span = span;
    }
    Stmt::Empty(s) => {
      s.span = span;
    }
    Stmt::Debugger(s) => {
      s.span = span;
    }
    Stmt::With(s) => {
      s.span = span;
    }
    Stmt::Return(s) => {
      s.span = span;
    }
    Stmt::Labeled(s) => {
      s.span = span;
    }
    Stmt::Break(s) => {
      s.span = span;
    }
    Stmt::Continue(s) => {
      s.span = span;
    }
    Stmt::If(s) => {
      s.span = span;
    }
    Stmt::Switch(s) => {
      s.span = span;
    }
    Stmt::Throw(s) => {
      s.span = span;
    }
    Stmt::Try(s) => {
      s.span = span;
    }
    Stmt::While(s) => {
      s.span = span;
    }
    Stmt::DoWhile(s) => {
      s.span = span;
    }
    Stmt::For(s) => {
      s.span = span;
    }
    Stmt::ForIn(s) => {
      s.span = span;
    }
    Stmt::ForOf(s) => {
      s.span = span;
    }
    Stmt::Decl(s) => (),
    Stmt::Expr(s) => {
      s.span = span;
    }
  }

  stmt
}
