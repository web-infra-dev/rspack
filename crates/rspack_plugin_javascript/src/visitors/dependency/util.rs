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

/// Match the expr is `import.meta.webpackHot`
pub fn is_expr_exact_import_meta_webpack_hot(expr: &Expr) -> bool {
  use swc_core::ecma::ast;
  match expr {
    ast::Expr::Member(ast::MemberExpr {
      obj:
        box ast::Expr::MetaProp(ast::MetaPropExpr {
          kind: ast::MetaPropKind::ImportMeta,
          ..
        }),
      prop: ast::MemberProp::Ident(ast::Ident { sym: prop, .. }),
      ..
    }) if prop == "webpackHot" => true,
    _ => false,
  }
}

pub fn is_member_expr_starts_with_import_meta_webpack_hot(expr: &Expr) -> bool {
  use swc_core::ecma::ast;
  let mut match_target = expr;

  loop {
    match match_target {
      // If the target self is `import.meta.webpackHot` just return true
      ast::Expr::Member(..) if is_expr_exact_import_meta_webpack_hot(match_target) => return true,
      // The expr is sub-part of `import.meta.webpackHot.xxx`. Recursively look up.
      ast::Expr::Member(ast::MemberExpr { obj, .. }) if obj.is_member() => {
        match_target = obj.as_ref();
      }
      // The expr could never be `import.meta.webpackHot`
      _ => return false,
    }
  }
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
  assert!(is_member_expr_starts_with_import_meta_webpack_hot(
    &import_meta_expr
  ));
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
