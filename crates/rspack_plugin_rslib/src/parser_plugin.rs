use rspack_plugin_javascript::{JavascriptParserPlugin, visitors::JavascriptParser};

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
  fn member(
    &self,
    _parser: &mut JavascriptParser,
    member_expr: &swc_core::ecma::ast::MemberExpr,
    _name: &str,
  ) -> Option<bool> {
    // Intercept `require.cache` of APIPlugin.
    if self.intercept_api_plugin
      && let swc_core::ecma::ast::Expr::Ident(ident) = &*member_expr.obj
    {
      let prop = &member_expr.prop;
      if let swc_core::ecma::ast::MemberProp::Ident(id) = prop
        && ident.sym == "require"
        && id.sym == "cache"
      {
        return Some(true);
      }
    }

    None
  }
}
