use rustc_hash::FxHashSet as HashSet;
use swc_core::{
  common::SyntaxContext,
  ecma::{
    ast::{CallExpr, Expr, MemberExpr, ObjectPat, ObjectPatProp, PropName},
    atoms::JsWord,
  },
};

pub fn collect_destructuring_assignment_properties(
  object_pat: &ObjectPat,
) -> Option<HashSet<JsWord>> {
  let mut properties = HashSet::default();

  for property in &object_pat.props {
    match property {
      ObjectPatProp::Assign(assign) => {
        properties.insert(assign.key.sym.clone());
      }
      ObjectPatProp::KeyValue(key_value) => {
        if let PropName::Ident(ident) = &key_value.key {
          properties.insert(ident.sym.clone());
        }
      }
      ObjectPatProp::Rest(_) => {}
    }
  }

  if properties.is_empty() {
    return None;
  }
  Some(properties)
}

pub(crate) mod expr_matcher {
  use std::sync::Arc;

  use once_cell::sync::Lazy;
  use swc_core::{
    common::{EqIgnoreSpan, SourceMap},
    ecma::{ast::Ident, parser::parse_file_as_expr},
  };

  static PARSED_MEMBER_EXPR_CM: Lazy<Arc<SourceMap>> = Lazy::new(Default::default);

  // The usage of define_member_expr_matchers is limited in `member_expr_matcher`.
  // Do not extends it's usage out of this mod.
  macro_rules! define_expr_matchers {
    ({
      $($fn_name:ident: $first:expr,)*
    }) => {
          use super::Expr;
          $(pub(crate) fn $fn_name(expr: &Expr) -> bool {
            static TARGET: Lazy<Box<Expr>> = Lazy::new(|| {
              let mut errors = vec![];
              let fm =
                 PARSED_MEMBER_EXPR_CM.new_source_file(swc_core::common::FileName::Anon, $first.to_string());
                 let expr = parse_file_as_expr(
                  &fm,
                  Default::default(),
                  Default::default(),
                  None,
                  &mut errors,
                )
                .unwrap_or_else(|_| panic!("Member matcher parsed failed {:?}", $first));
                assert!(errors.is_empty());
                expr
            });
            Ident::within_ignored_ctxt(|| {
              (&**TARGET).eq_ignore_span(expr)
            })
          })+

      };
  }

  // Notice:
  // - `import.meta` is not a MemberExpr
  // - `import.meta.xxx` is a MemberExpr
  // - Matching would ignore Span and SyntaxContext
  define_expr_matchers!({
    is_require: "require",
    is_require_main: "require.main",
    is_require_context: "require.context",
    is_require_resolve: "require.resolve",
    is_require_resolve_weak: "require.resolveWeak",
    is_require_cache: "require.cache",
    is_module_hot_accept: "module.hot.accept",
    is_module_hot_decline: "module.hot.decline",
    is_module_hot: "module.hot",
    is_module_id: "module.id",
    is_module_loaded: "module.loaded",
    is_module_exports: "module.exports",
    is_module_require: "module.require",
    is_webpack_module_id: "__webpack_module__.id",
    is_import_meta_webpack_hot: "import.meta.webpackHot",
    is_import_meta_webpack_hot_accept: "import.meta.webpackHot.accept",
    is_import_meta_webpack_hot_decline: "import.meta.webpackHot.decline",
    is_import_meta_webpack_context: "import.meta.webpackContext",
    is_import_meta_url: "import.meta.url",
    is_import_meta: "import.meta",
    is_object_define_property: "Object.defineProperty",
  });
}

pub fn is_require_call_expr(expr: &Expr, ctxt: SyntaxContext) -> bool {
  matches!(expr, Expr::Call(call_expr) if is_require_call(call_expr, ctxt))
}

pub fn is_require_call(node: &CallExpr, ctxt: SyntaxContext) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| matches!(expr, box Expr::Ident(ident) if &ident.sym == "require" && ident.span.ctxt == ctxt))
    .unwrap_or_default()
}

pub fn is_require_context_call(node: &CallExpr) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| expr_matcher::is_require_context(expr))
    .unwrap_or_default()
}

pub fn is_require_resolve_call(node: &CallExpr) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| expr_matcher::is_require_resolve(expr))
    .unwrap_or_default()
}

pub fn is_require_resolve_weak_call(node: &CallExpr) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| expr_matcher::is_require_resolve_weak(expr))
    .unwrap_or_default()
}

pub fn is_module_hot_accept_call(node: &CallExpr) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| expr_matcher::is_module_hot_accept(expr))
    .unwrap_or_default()
}

pub fn is_module_hot_decline_call(node: &CallExpr) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| expr_matcher::is_module_hot_decline(expr))
    .unwrap_or_default()
}

pub fn is_import_meta_hot_accept_call(node: &CallExpr) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| expr_matcher::is_import_meta_webpack_hot_accept(expr))
    .unwrap_or_default()
}

pub fn is_import_meta_hot_decline_call(node: &CallExpr) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| expr_matcher::is_import_meta_webpack_hot_decline(expr))
    .unwrap_or_default()
}

pub fn is_import_meta_context_call(node: &CallExpr) -> bool {
  node
    .callee
    .as_expr()
    .map(|expr| expr_matcher::is_import_meta_webpack_context(expr))
    .unwrap_or_default()
}

pub fn is_member_expr_starts_with<F>(mut expr: &Expr, checker: F) -> bool
where
  F: Fn(&Expr) -> bool,
{
  loop {
    match expr {
      _ if checker(expr) => return true,
      Expr::Member(MemberExpr { obj, .. }) => expr = obj.as_ref(),
      _ => return false,
    }
  }
}

// Notice: Include `import.meta.webpackHot` itself
pub fn is_member_expr_starts_with_import_meta_webpack_hot(expr: &Expr) -> bool {
  use swc_core::ecma::ast;
  let mut match_target = expr;

  loop {
    match match_target {
      // If the target self is `import.meta.webpackHot` just return true
      ast::Expr::Member(..) if expr_matcher::is_import_meta_webpack_hot(match_target) => {
        return true
      }
      // The expr is sub-part of `import.meta.webpackHot.xxx`. Recursively look up.
      ast::Expr::Member(ast::MemberExpr { obj, .. }) if obj.is_member() => {
        match_target = obj.as_ref();
      }
      // The expr could never be `import.meta.webpackHot`
      _ => return false,
    }
  }
}

pub fn parse_order_string(x: &str) -> Option<u32> {
  match x {
    "true" => Some(0),
    "false" => None,
    _ => {
      if let Ok(order) = x.parse::<u32>() {
        Some(order)
      } else {
        None
      }
    }
  }
}

#[test]
fn test() {
  use swc_core::common::DUMMY_SP;
  use swc_core::ecma::ast::{Ident, MemberExpr, MemberProp, MetaPropExpr, MetaPropKind};
  use swc_core::ecma::utils::member_expr;
  use swc_core::ecma::utils::ExprFactory;
  let expr = *member_expr!(DUMMY_SP, module.hot.accept);
  assert!(expr_matcher::is_module_hot_accept(&expr));
  assert!(!expr_matcher::is_module_hot_decline(&expr));
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
  assert!(is_member_expr_starts_with(
    &import_meta_expr,
    |expr: &Expr| expr_matcher::is_import_meta(expr)
  ));
  assert!(is_member_expr_starts_with_import_meta_webpack_hot(
    &import_meta_expr
  ));
  assert!(expr_matcher::is_import_meta_webpack_hot_accept(
    &import_meta_expr,
  ));
  assert!(is_import_meta_hot_accept_call(&CallExpr {
    span: DUMMY_SP,
    callee: import_meta_expr.as_callee(),
    args: vec![],
    type_args: None
  }));
}

pub fn is_unresolved_member_object_ident(expr: &Expr, unresolved_ctxt: SyntaxContext) -> bool {
  if let Expr::Member(member) = expr {
    if let Expr::Ident(ident) = &*member.obj {
      return ident.span.ctxt == unresolved_ctxt;
    };
  }
  false
}

pub fn is_unresolved_require(expr: &Expr, unresolved_ctxt: SyntaxContext) -> bool {
  let ident = match expr {
    Expr::Ident(ident) => Some(ident),
    Expr::Member(mem) => mem.obj.as_ident(),
    _ => None,
  };
  let Some(ident) = ident else {
    unreachable!("please don't use this fn in other case");
  };
  assert!(ident.sym.eq("require"));
  ident.span.ctxt == unresolved_ctxt
}

#[macro_export]
macro_rules! no_visit_ignored_stmt {
  () => {
    fn visit_stmt(&mut self, stmt: &swc_core::ecma::ast::Stmt) {
      use rspack_core::SpanExt;
      use swc_core::common::Spanned;
      use swc_core::ecma::visit::VisitWith;
      let span = stmt.span();
      if self
        .ignored
        .iter()
        .any(|r| r.start() <= span.real_lo() && span.real_hi() <= r.end())
      {
        return;
      }
      stmt.visit_children_with(self);
    }
  };
}

#[macro_export]
macro_rules! no_visit_ignored_expr {
  () => {
    fn visit_expr(&mut self, expr: &swc_core::ecma::ast::Expr) {
      use rspack_core::SpanExt;
      use swc_core::common::Spanned;
      use swc_core::ecma::visit::VisitWith;
      let span = expr.span();
      if self
        .ignored
        .iter()
        .any(|r| r.start() <= span.real_lo() && span.real_hi() <= r.end())
      {
        return;
      }
      expr.visit_children_with(self);
    }
  };
}
