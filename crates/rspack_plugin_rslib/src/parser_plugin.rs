use rspack_plugin_javascript::{JavascriptParserPlugin, visitors::JavascriptParser};
use swc_experimental_ecma_ast::{MemberExpr, UnaryExpr};

#[derive(PartialEq, Debug, Default)]
pub struct RslibParserPlugin {
  intercept_api_plugin: bool,
}

impl RslibParserPlugin {
  pub fn new(intercept_api_plugin: bool) -> Self {
    Self {
      intercept_api_plugin,
    }
  }
}

#[rspack_plugin_javascript::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for RslibParserPlugin {
  fn member(
    &self,
    _parser: &mut JavascriptParser<'p>,
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
    _parser: &mut JavascriptParser<'p>,
    _expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "module" {
      Some(false)
    } else {
      None
    }
  }
}
