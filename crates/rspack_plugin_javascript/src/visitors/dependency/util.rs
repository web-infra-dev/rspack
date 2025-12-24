use rspack_core::DependencyRange;
use rspack_error::{Diagnostic, Error, Severity};
use rspack_regex::RspackRegex;
use swc_core::{
  atoms::Atom,
  ecma::ast::{Expr, MemberExpr, OptChainBase},
};

use super::JavascriptParser;

pub mod expr_name {
  pub const MODULE: &str = "module";
  pub const MODULE_HOT: &str = "module.hot";
  pub const MODULE_HOT_ACCEPT: &str = "module.hot.accept";
  pub const MODULE_HOT_DECLINE: &str = "module.hot.decline";
  pub const MODULE_REQUIRE: &str = "module.require";
  pub const REQUIRE: &str = "require";
  pub const REQUIRE_RESOLVE: &str = "require.resolve";
  pub const REQUIRE_RESOLVE_WEAK: &str = "require.resolveWeak";
  pub const IMPORT_META: &str = "import.meta";
  pub const IMPORT_META_FILENAME: &str = "import.meta.filename";
  pub const IMPORT_META_DIRNAME: &str = "import.meta.dirname";
  pub const IMPORT_META_URL: &str = "import.meta.url";
  pub const IMPORT_META_RESOLVE: &str = "import.meta.resolve";
  pub const IMPORT_META_VERSION: &str = "import.meta.webpack";
  pub const IMPORT_META_HOT: &str = "import.meta.webpackHot";
  pub const IMPORT_META_HOT_ACCEPT: &str = "import.meta.webpackHot.accept";
  pub const IMPORT_META_HOT_DECLINE: &str = "import.meta.webpackHot.decline";
  pub const IMPORT_META_CONTEXT: &str = "import.meta.webpackContext";
}

pub fn parse_order_string(x: &str) -> Option<i32> {
  match x {
    "true" => Some(0),
    "false" => None,
    _ => x.parse::<i32>().ok(),
  }
}

pub fn create_traceable_error(
  title: String,
  message: String,
  source: String,
  span: DependencyRange,
) -> Error {
  Error::from_string(
    Some(source),
    span.start as usize,
    span.end as usize,
    title,
    message,
  )
}

pub fn context_reg_exp(
  expr: &str,
  flags: &str,
  error_span: Option<DependencyRange>,
  parser: &mut JavascriptParser,
) -> Option<RspackRegex> {
  if expr.is_empty() {
    return None;
  }
  let regexp = RspackRegex::with_flags(expr, flags).expect("reg failed");
  clean_regexp_in_context_module(regexp, error_span, parser)
}

pub fn clean_regexp_in_context_module(
  regexp: RspackRegex,
  error_span: Option<DependencyRange>,
  parser: &mut JavascriptParser,
) -> Option<RspackRegex> {
  if regexp.sticky() || regexp.global() {
    if let Some(error_span) = error_span {
      let mut error = create_traceable_error(
        "Critical dependency".into(),
        "Contexts can't use RegExps with the 'g' or 'y' flags".to_string(),
        parser.source.to_owned(),
        error_span,
      );
      error.severity = Severity::Warning;
      parser.add_warning(Diagnostic::from(error));
    } else {
      let mut err = Error::warning("Contexts can't use RegExps with the 'g' or 'y' flags".into());
      err.code = Some("Critical dependency".into());
      parser.add_warning(err.into());
    }
    None
  } else {
    Some(regexp)
  }
}

pub fn get_non_optional_part<'a>(members: &'a [Atom], members_optionals: &[bool]) -> &'a [Atom] {
  let mut i = 0;
  while i < members.len() && matches!(members_optionals.get(i), Some(false)) {
    i += 1;
  }
  if i != members.len() {
    &members[0..i]
  } else {
    members
  }
}

pub fn get_non_optional_member_chain_from_expr(mut expr: &Expr, mut count: i32) -> &Expr {
  while count != 0 {
    if let Expr::Member(member) = expr {
      expr = &member.obj;
      count -= 1;
    } else if let Expr::OptChain(opt_chain) = expr {
      expr = match &*opt_chain.base {
        OptChainBase::Member(member) => &*member.obj,
        OptChainBase::Call(call) if call.callee.as_member().is_some() => {
          let member = call
            .callee
            .as_member()
            .expect("`call.callee` is `MemberExpr` in `if_guard`");
          &*member.obj
        }
        _ => unreachable!(),
      };
      count -= 1;
    } else {
      unreachable!()
    }
  }
  expr
}

pub fn get_non_optional_member_chain_from_member(member: &MemberExpr, mut count: i32) -> &Expr {
  count -= 1;
  get_non_optional_member_chain_from_expr(&member.obj, count)
}
