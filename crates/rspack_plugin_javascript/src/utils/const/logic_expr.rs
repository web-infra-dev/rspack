use rspack_core::ConstDependency;
use rspack_core::SpanExt;
use swc_core::common::Spanned;
use swc_core::ecma::ast::{BinExpr, BinaryOp};

use crate::visitors::common_js_import_dependency_scanner::CommonJsImportDependencyScanner;

// FIXME: a temp hack to avoid bwchecker.
bitflags::bitflags! {
  pub struct Continue: u8 {
    const LEFT = 1 << 0;
    const RIGHT = 1 << 1;
  }
}

pub fn expression_logic_operator(
  scanner: &CommonJsImportDependencyScanner<'_>,
  expr: &BinExpr,
) -> (Option<Vec<ConstDependency>>, Continue) {
  if expr.op == BinaryOp::LogicalAnd || expr.op == BinaryOp::LogicalOr {
    let param = scanner.evaluate_expression(&expr.left);
    let boolean = param.as_bool();
    let Some(bool) = boolean else {
      return (None, Continue::all());
    };
    let mut deps = vec![];
    let mut c = Continue::empty();
    let keep_right = if bool {
      expr.op == BinaryOp::LogicalAnd
    } else {
      expr.op == BinaryOp::LogicalOr
    };
    if !param.could_have_side_effects() && (keep_right || param.is_bool()) {
      let dep = ConstDependency::new(
        param.range().0,
        param.range().1 - 1,
        format!(" {bool}").into(),
        None,
      );
      deps.push(dep);
    } else {
      c.insert(Continue::LEFT);
    }

    if !keep_right {
      let dep = ConstDependency::new(
        expr.right.span().real_lo(),
        expr.right.span().hi().0 - 1,
        "0".into(),
        None,
      );
      deps.push(dep);
    } else {
      c.insert(Continue::RIGHT)
    }
    (Some(deps), c)
  } else {
    (None, Continue::all())
  }
  // else if expr.op == BinaryOp::NullishCoalescing {
  //   // TODO: support `??`
  //   expr.visit_children_with(scanner);
  //   None
  // }
}
