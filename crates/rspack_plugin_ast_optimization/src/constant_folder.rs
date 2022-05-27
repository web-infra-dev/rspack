use rspack_swc::{
  swc_common::Mark,
  swc_ecma_ast::{op, Bool, Expr, Lit, Stmt},
  // swc_ecma_transforms_optimization::simplify::dead_branch_remover,
  swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
  swc_plugin::{chain, utils::take::Take},
};

use crate::utils::replace_span_for_stmt;

struct BinarySimplifier;

impl VisitMut for BinarySimplifier {
  noop_visit_mut_type!();

  fn visit_mut_expr(&mut self, n: &mut Expr) {
    if let Expr::Bin(b) = n {
      let left_lit = &*b.left;
      let right_lit = &*b.right;

      let mut left_str = None;
      let mut right_str = None;

      if let Expr::Lit(Lit::Str(s)) = left_lit {
        left_str = Some(s.value.as_ref())
      }

      if let Expr::Lit(Lit::Str(s)) = right_lit {
        right_str = Some(s.value.as_ref())
      }

      if left_str.is_some() && right_str.is_some() {
        match b.op {
          op!("===") | op!("==") => {
            *n = Expr::Lit(Lit::Bool(Bool {
              span: b.span,
              value: left_str == right_str,
            }));
          }
          op!("!==") | op!("!=") => {
            *n = Expr::Lit(Lit::Bool(Bool {
              span: b.span,
              value: left_str != right_str,
            }));
          }
          _ => (),
        }
      }
    }

    n.visit_mut_children_with(self);
  }
}

struct DeadBranchRemover;

impl VisitMut for DeadBranchRemover {
  noop_visit_mut_type!();

  fn visit_mut_stmt(&mut self, n: &mut Stmt) {
    if let Stmt::If(if_stmt) = n {
      let if_span = if_stmt.span;
      if let Expr::Lit(Lit::Bool(literal)) = &*if_stmt.test {
        if literal.value {
          *n = replace_span_for_stmt(*if_stmt.cons.take(), if_span);
        } else if let Some(s) = if_stmt.alt.take().map(|x| *x) {
          *n = replace_span_for_stmt(s, if_span);
        }
      }
    }

    n.visit_mut_children_with(self);
  }
}

// TODO: distinguish prod and dev
pub fn constant_folder(_unresolved_mark: Mark) -> impl Fold + 'static {
  chain!(
    as_folder(BinarySimplifier),
    as_folder(DeadBranchRemover) // dead_branch_remover(unresolved_mark)
  )
}
