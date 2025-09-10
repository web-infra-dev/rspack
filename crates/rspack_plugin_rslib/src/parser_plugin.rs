use rspack_plugin_javascript::{JavascriptParserPlugin, visitors::JavascriptParser};
use swc_core::ecma::ast::Ident;

#[derive(PartialEq, Debug, Default)]
pub struct RslibParserPlugin {
  pub intercept_api_plugin: bool,
}

impl RslibParserPlugin {
  pub fn new(intercept_api_plugin: bool) -> Self {
    Self {
      intercept_api_plugin,
    }
  }
}

impl JavascriptParserPlugin for RslibParserPlugin {
  fn identifier(
    &self,
    _parser: &mut JavascriptParser,
    _ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    // Intercept CommonJsExportsParsePlugin, not APIPlugin, but put it here.
    // crates/rspack_plugin_javascript/src/parser_plugin/common_js_exports_parse_plugin.rs
    if for_name == "module" {
      return Some(true);
    }

    None
  }

  fn member(
    &self,
    _parser: &mut JavascriptParser,
    _member_expr: &swc_core::ecma::ast::MemberExpr,
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
}
