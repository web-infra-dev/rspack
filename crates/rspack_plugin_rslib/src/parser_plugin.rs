use rspack_plugin_javascript::{
  JavascriptParserPlugin,
  visitors::{JavascriptParser, extract_member_root},
};

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
    let expr = &swc_core::ecma::ast::Expr::Member(member_expr.to_owned());
    // Intercept APIPlugin.
    if self.intercept_api_plugin
      && let Some(root) = extract_member_root(expr)
    {
      let prop = &member_expr.prop;
      if let swc_core::ecma::ast::MemberProp::Ident(id) = prop
        && root.sym == "require"
        && (id.sym == "cache"
          // not_supported_expr
            || id.sym == "extensions"
            || id.sym == "config"
            || id.sym == "version"
            || id.sym == "include"
            || id.sym == "onError")
      {
        return Some(true);
      }
    }

    None
  }
}
