use swc_core::{
  common::pass::AstNodePath,
  ecma::{
    ast::{CallExpr, Expr, MemberProp, MetaPropKind},
    visit::{AstParentKind, AstParentNodeRef},
  },
};

pub fn as_parent_path(ast_path: &AstNodePath<AstParentNodeRef<'_>>) -> Vec<AstParentKind> {
  ast_path.iter().map(|n| n.kind()).collect()
}

pub fn match_member_expr(mut expr: &Expr, value: &str) -> bool {
  let mut parts = value.split('.');
  let first = parts.next().expect("should have a last str");
  for part in parts.rev() {
    if let Expr::Member(member_expr) = expr {
      if let MemberProp::Ident(ident) = &member_expr.prop {
        if ident.sym.eq(part) {
          expr = &member_expr.obj;
          continue;
        }
      }
    }
    return false;
  }
  matches!(&expr, Expr::Ident(ident) if ident.sym.eq(first))
}

#[inline]
fn is_hmr_api_call(node: &CallExpr, value: &str) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| match_member_expr(expr, value))
    .unwrap_or_default()
}

pub fn is_require_context_call(node: &CallExpr) -> bool {
  is_hmr_api_call(node, "require.context")
}

pub fn is_module_hot_accept_call(node: &CallExpr) -> bool {
  is_hmr_api_call(node, "module.hot.accept")
}

pub fn is_module_hot_decline_call(node: &CallExpr) -> bool {
  is_hmr_api_call(node, "module.hot.decline")
}

pub fn match_import_meta_member_expr(mut expr: &Expr, value: &str) -> bool {
  let mut parts = value.split('.');
  // pop import.meta
  parts.next();
  parts.next();
  for part in parts.rev() {
    if let Expr::Member(member_expr) = expr {
      if let MemberProp::Ident(ident) = &member_expr.prop {
        if ident.sym.eq(part) {
          expr = &member_expr.obj;
          continue;
        }
      }
    }
    return false;
  }
  is_import_meta(expr)
}

#[inline]
pub fn is_import_meta(expr: &Expr) -> bool {
  matches!(&expr, Expr::MetaProp(meta) if meta.kind == MetaPropKind::ImportMeta)
}

pub fn is_import_meta_member_expr(expr: &Expr) -> bool {
  fn valid_member_expr_obj(expr: &Expr) -> bool {
    if is_import_meta(expr) {
      return true;
    }
    is_import_meta_member_expr(expr)
  }

  if let Expr::Member(member_expr) = expr {
    return valid_member_expr_obj(&member_expr.obj);
  }
  false
}

fn is_hmr_import_meta_api_call(node: &CallExpr, value: &str) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| match_import_meta_member_expr(expr, value))
    .unwrap_or_default()
}

pub fn is_import_meta_hot_accept_call(node: &CallExpr) -> bool {
  is_hmr_import_meta_api_call(node, "import.meta.webpackHot.accept")
}

pub fn is_import_meta_hot_decline_call(node: &CallExpr) -> bool {
  is_hmr_import_meta_api_call(node, "import.meta.webpackHot.decline")
}

pub fn is_import_meta_hot(expr: &Expr) -> bool {
  let v = member_expr_to_string(expr);
  v.starts_with("import.meta.webpackHot")
}

pub fn member_expr_to_string(expr: &Expr) -> String {
  fn collect_member_expr(expr: &Expr, arr: &mut Vec<String>) {
    if let Expr::Member(member_expr) = expr {
      if let MemberProp::Ident(ident) = &member_expr.prop {
        arr.push(ident.sym.to_string());
      }
      collect_member_expr(&member_expr.obj, arr);
    }
    // add length check to improve performance, avoid match extra expr
    if arr.is_empty() {
      return;
    }
    if is_import_meta(expr) {
      arr.push("meta".to_string());
      arr.push("import".to_string());
    }
    if let Expr::Ident(ident) = expr {
      arr.push(ident.sym.to_string());
    }
  }

  let mut res = vec![];
  collect_member_expr(expr, &mut res);
  res.reverse();
  res.join(".")
}

#[test]
fn test() {
  use swc_core::common::DUMMY_SP;
  use swc_core::ecma::ast::{Ident, MemberExpr, MetaPropExpr};
  use swc_core::ecma::utils::member_expr;
  use swc_core::ecma::utils::ExprFactory;
  let expr = *member_expr!(DUMMY_SP, module.hot.accept);
  assert!(match_member_expr(&expr, "module.hot.accept"));
  assert!(is_module_hot_accept_call(&CallExpr {
    span: DUMMY_SP,
    callee: expr.as_callee(),
    args: vec![],
    type_args: None
  }));

  let import_meta_expr = Expr::Member(MemberExpr {
    span: DUMMY_SP,
    obj: Box::new(Expr::Member(MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(Expr::MetaProp(MetaPropExpr {
        span: DUMMY_SP,
        kind: MetaPropKind::ImportMeta,
      })),
      prop: MemberProp::Ident(Ident::new("webpackHot".into(), DUMMY_SP)),
    })),
    prop: MemberProp::Ident(Ident::new("accept".into(), DUMMY_SP)),
  });
  assert!(is_import_meta_member_expr(&import_meta_expr));
  assert!(is_import_meta_hot(&import_meta_expr));
  assert!(match_import_meta_member_expr(
    &import_meta_expr,
    "import.meta.webpackHot.accept"
  ));
  assert!(is_import_meta_hot_accept_call(&CallExpr {
    span: DUMMY_SP,
    callee: import_meta_expr.as_callee(),
    args: vec![],
    type_args: None
  }));
}
