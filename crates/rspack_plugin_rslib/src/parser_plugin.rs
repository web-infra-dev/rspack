use rspack_plugin_javascript::{JavascriptParserPlugin, visitors::JavascriptParser};
use swc_core::ecma::ast::MemberExpr;

#[derive(PartialEq, Debug, Default)]
pub(crate) struct RslibParserPlugin {
  pub intercept_api_plugin: bool,
}

impl RslibParserPlugin {
  pub(crate) fn new(intercept_api_plugin: bool) -> Self {
    Self {
      intercept_api_plugin,
    }
  }
}

impl JavascriptParserPlugin for RslibParserPlugin {
  fn member(
    &self,
    _parser: &mut JavascriptParser,
    _member_expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "require.cache"
      || for_name == "require.extensions"
      || for_name == "require.config"
      || for_name == "require.version"
      || for_name == "require.include"
      || for_name == "require.onError"
    {
      return Some(true);
    }
    None
  }

  fn r#typeof(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "module" {
      Some(false)
    } else {
      None
    }
  }
}
