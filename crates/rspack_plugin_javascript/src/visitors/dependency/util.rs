use itertools::Itertools;
use rspack_core::extract_member_expression_chain;
use rspack_core::{ConstDependency, DependencyLocation, ErrorSpan, ExpressionInfoKind, SpanExt};
use rspack_error::{miette::Severity, DiagnosticKind, TraceableError};
use rustc_hash::FxHashSet as HashSet;
use swc_core::common::{SourceFile, Spanned};
use swc_core::ecma::ast::*;
use swc_core::ecma::atoms::Atom;

pub fn collect_destructuring_assignment_properties(
  object_pat: &ObjectPat,
) -> Option<HashSet<Atom>> {
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
    None
  } else {
    Some(properties)
  }
}

pub(crate) mod expr_like {
  use std::any::Any;

  use rspack_util::ext::AsAny;
  use swc_core::common::EqIgnoreSpan;
  use swc_core::ecma::ast::{Expr, Ident, MemberExpr, ThisExpr};

  pub trait DynEqIgnoreSpan: __::Sealed {
    fn dyn_eq_ignore_span(&self, other: &dyn Any) -> bool;
  }
  impl<T: EqIgnoreSpan + ExprLike + Any> DynEqIgnoreSpan for T {
    fn dyn_eq_ignore_span(&self, other: &dyn Any) -> bool {
      if let Some(other) = other.downcast_ref::<T>() {
        self.eq_ignore_span(other)
      } else {
        false
      }
    }
  }

  pub trait ExprLikeEqIgnoreSpan: __::Sealed {
    fn expr_like_eq_ignore_span(&self, other: &dyn ExprLike) -> bool;
  }
  impl ExprLikeEqIgnoreSpan for dyn ExprLike {
    fn expr_like_eq_ignore_span(&self, other: &dyn ExprLike) -> bool {
      match (self, other) {
        (left, right)
          if let Some(left) = left.as_member()
            && let Some(right) = right.as_member() =>
        {
          left.dyn_eq_ignore_span(right)
        }
        (left, right)
          if let Some(left) = left.as_ident()
            && let Some(right) = right.as_ident() =>
        {
          left.dyn_eq_ignore_span(right)
        }
        _ => false,
      }
    }
  }

  impl PartialEq for dyn ExprLike + '_ {
    fn eq(&self, other: &Self) -> bool {
      self.dyn_eq_ignore_span(other.as_any())
    }
  }

  mod __ {
    pub trait Sealed {}
  }

  macro_rules! expr_like {
    ($(($ident:ident,$ty:ty),)*) => {
      use std::fmt::Debug;
      pub(crate) trait ExprLike: 'static + __::Sealed + Debug + Send + Sync + DynEqIgnoreSpan + AsAny {
        fn as_expr(&self) -> Option<&Expr> {
          None
        }
        $(
          fn $ident(&self) -> Option<&$ty> {
            None
          }
        )*
      }
      $(
        impl ExprLike for $ty {
          fn $ident(&self) -> Option<&$ty> {
            Some(self)
          }
        }
        impl __::Sealed for $ty {}
      )*
    }
  }

  expr_like! {
    (as_member, MemberExpr),
    (as_this, ThisExpr),
    (as_ident, Ident),
  }

  impl ExprLike for Expr {
    fn as_expr(&self) -> Option<&Expr> {
      Some(self)
    }

    fn as_member(&self) -> Option<&MemberExpr> {
      (*self).as_member()
    }

    fn as_this(&self) -> Option<&ThisExpr> {
      (*self).as_this()
    }

    fn as_ident(&self) -> Option<&Ident> {
      (*self).as_ident()
    }
  }
  impl __::Sealed for Expr {}
}

pub(crate) mod expr_matcher {
  use std::sync::Arc;

  use once_cell::sync::Lazy;
  use swc_core::common::SourceMap;
  use swc_core::ecma::{ast::Ident, parser::parse_file_as_expr};

  use super::expr_like::*;

  static PARSED_MEMBER_EXPR_CM: Lazy<Arc<SourceMap>> = Lazy::new(Default::default);

  // The usage of define_member_expr_matchers is limited in `member_expr_matcher`.
  // Do not extends it's usage out of this mod.
  macro_rules! define_expr_matchers {
    ({
      $($fn_name:ident: $first:expr,)*
    }) => {
          $(pub(crate) fn $fn_name<E: ExprLike>(expr: &E) -> bool {
            static TARGET: Lazy<Box<dyn ExprLike>> = Lazy::new(|| {
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
              TARGET.expr_like_eq_ignore_span(expr)
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
    is_require_cache: "require.cache",
    is_module: "module",
    is_module_id: "module.id",
    is_module_loaded: "module.loaded",
    is_module_exports: "module.exports",
    is_module_require: "module.require",
    is_webpack_module_id: "__webpack_module__.id",
    is_object_define_property: "Object.defineProperty",
    // unsupported
    is_require_extensions: "require.extensions",
    is_require_ensure: "require.ensure",
    is_require_config: "require.config",
    is_require_version: "require.version",
    is_require_amd: "require.amd",
    is_require_include: "require.include",
    is_require_onerror: "require.onError",
    is_require_main_require: "require.main.require",
    is_module_parent_require: "module.parent.require",
  });
}

pub mod expr_name {
  pub const MODULE: &str = "module";
  pub const MODULE_HOT: &str = "module.hot";
  pub const MODULE_HOT_ACCEPT: &str = "module.hot.accept";
  pub const MODULE_HOT_DECLINE: &str = "module.hot.decline";
  pub const REQUIRE: &str = "require";
  pub const REQUIRE_RESOLVE: &str = "require.resolve";
  pub const REQUIRE_RESOLVE_WEAK: &str = "require.resolveWeak";
  pub const IMPORT_META: &str = "import.meta";
  pub const IMPORT_META_URL: &str = "import.meta.url";
  pub const IMPORT_META_WEBPACK_HOT: &str = "import.meta.webpackHot";
  pub const IMPORT_META_WEBPACK_HOT_ACCEPT: &str = "import.meta.webpackHot.accept";
  pub const IMPORT_META_WEBPACK_HOT_DECLINE: &str = "import.meta.webpackHot.decline";
  pub const IMPORT_META_WEBPACK_CONTEXT: &str = "import.meta.webpackContext";
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

pub fn extract_require_call_info(
  expr: &Expr,
) -> Option<(Vec<Atom>, ExprOrSpread, DependencyLocation)> {
  let member_info = extract_member_expression_chain(expr);
  let root_members = match member_info.kind() {
    ExpressionInfoKind::CallExpression(info) => info
      .root_members()
      .iter()
      .map(|n| n.0.to_owned())
      .collect_vec(),
    ExpressionInfoKind::MemberExpression(_) => vec![],
    ExpressionInfoKind::Expression => vec![],
  };
  let args = match member_info.kind() {
    ExpressionInfoKind::CallExpression(info) => {
      info.args().iter().map(|i| i.to_owned()).collect_vec()
    }
    ExpressionInfoKind::MemberExpression(_) => vec![],
    ExpressionInfoKind::Expression => vec![],
  };

  let members = member_info
    .members()
    .iter()
    .map(|n| n.0.to_owned())
    .collect_vec();

  let Some(fist_arg) = args.first() else {
    return None;
  };

  let loc = DependencyLocation::new(expr.span().real_lo(), expr.span().real_hi());

  if (root_members.len() == 1 && root_members.first().is_some_and(|f| f == "require"))
    || (root_members.len() == 2
      && root_members.first().is_some_and(|f| f == "module")
      && root_members.get(1).is_some_and(|f| f == "require"))
  {
    Some((members, fist_arg.to_owned(), loc))
  } else {
    None
  }
}

pub fn is_require_call_start(expr: &Expr) -> bool {
  match expr {
    Expr::Call(CallExpr { callee, .. }) => {
      return callee
        .as_expr()
        .map(|callee| {
          if expr_matcher::is_require(&**callee) || expr_matcher::is_module_require(&**callee) {
            true
          } else {
            is_require_call_start(callee)
          }
        })
        .unwrap_or(false);
    }
    Expr::Member(MemberExpr { obj, .. }) => is_require_call_start(obj),
    _ => false,
  }
}

pub fn expression_not_supported(
  file: &SourceFile,
  name: &str,
  expr: &Expr,
) -> (Box<TraceableError>, Box<ConstDependency>) {
  (
    Box::new(
      create_traceable_error(
        "Module parse failed".into(),
        format!("{name} is not supported by Rspack."),
        file,
        expr.span().into(),
      )
      .with_severity(Severity::Warning),
    ),
    Box::new(ConstDependency::new(
      expr.span().real_lo(),
      expr.span().real_hi(),
      "(void 0)".into(),
      None,
    )),
  )
}

pub fn extract_member_root(mut expr: &Expr) -> Option<Ident> {
  loop {
    match expr {
      Expr::Ident(id) => return Some(id.to_owned()),
      Expr::Member(MemberExpr { obj, .. }) => expr = obj.as_ref(),
      _ => return None,
    }
  }
}

pub fn create_traceable_error(
  title: String,
  message: String,
  fm: &SourceFile,
  span: ErrorSpan,
) -> TraceableError {
  TraceableError::from_source_file(fm, span.start as usize, span.end as usize, title, message)
    .with_kind(DiagnosticKind::JavaScript)
}

#[cfg(test)]
mod test {
  use swc_core::common::DUMMY_SP;

  use super::*;

  #[test]
  fn test_is_require_call_start() {
    macro_rules! test {
      ($tt:tt,$literal:literal) => {{
        let info = is_require_call_start(&swc_core::quote!($tt as Expr));
        assert_eq!(info, $literal)
      }};
    }
    test!("require().a.b", true);
    test!("require().a.b().c", true);
    test!("require()()", true);
    test!("require.a().b", false);
    test!("require.a.b", false);
    test!("a.require.b", false);
    test!("module.require().a.b", true);
    test!("module.require().a.b().c", true);
    test!("module.require()()", true);
    test!("module.require.a().b", false);
    test!("module.require.a.b", false);
    test!("a.module.require.b", false);
  }

  #[test]
  fn supports_expr_like() {
    let e = MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(
        Ident {
          span: DUMMY_SP,
          sym: "module".into(),
          optional: false,
        }
        .into(),
      ),
      prop: MemberProp::Ident(Ident {
        span: DUMMY_SP,
        sym: "exports".into(),
        optional: false,
      }),
    };
    assert!(
      expr_matcher::is_module_exports(&e),
      "should support evaluate with `MemberExpr`"
    );
    assert!(
      expr_matcher::is_module_exports(&Expr::Member(e)),
      "should support evaluate with `Expr::Member(MemberExpr {{ .. }})`"
    );

    let e = Ident {
      span: DUMMY_SP,
      sym: "module".into(),
      optional: false,
    };
    assert!(
      expr_matcher::is_module(&e),
      "should support evaluate with `Ident`"
    );
    assert!(
      expr_matcher::is_module(&Expr::Ident(e)),
      "should support evaluate with `Expr::Ident(Ident {{ .. }})`"
    );
  }
}
