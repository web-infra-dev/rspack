use rspack_core::DependencyRange;
use rspack_error::{
  TraceableError,
  miette::{Severity, diagnostic},
};
use rspack_regex::RspackRegex;
use swc_core::common::SourceFile;

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
  pub const IMPORT_META_URL: &str = "import.meta.url";
  pub const IMPORT_META_WEBPACK: &str = "import.meta.webpack";
  pub const IMPORT_META_WEBPACK_HOT: &str = "import.meta.webpackHot";
  pub const IMPORT_META_WEBPACK_HOT_ACCEPT: &str = "import.meta.webpackHot.accept";
  pub const IMPORT_META_WEBPACK_HOT_DECLINE: &str = "import.meta.webpackHot.decline";
  pub const IMPORT_META_WEBPACK_CONTEXT: &str = "import.meta.webpackContext";
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
  fm: &SourceFile,
  span: DependencyRange,
) -> TraceableError {
  TraceableError::from_source_file(fm, span.start as usize, span.end as usize, title, message)
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
      parser.warning_diagnostics.push(Box::new(
        create_traceable_error(
          "Critical dependency".into(),
          "Contexts can't use RegExps with the 'g' or 'y' flags".to_string(),
          parser.source_file,
          error_span,
        )
        .with_severity(rspack_error::RspackSeverity::Warn),
      ));
    } else {
      parser.warning_diagnostics.push(
        diagnostic!(
          severity = Severity::Warning,
          code = "Critical dependency",
          "Contexts can't use RegExps with the 'g' or 'y' flags"
        )
        .into(),
      );
    }
    None
  } else {
    Some(regexp)
  }
}
