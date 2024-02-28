use super::inner_graph::InnerGraphMapUsage;
use super::JavascriptParserPlugin;

pub struct JavascriptMetaInfoPlugin;

impl JavascriptParserPlugin for JavascriptMetaInfoPlugin {
  fn call(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    _expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "eval" {
      if let Some(current_symbol) = parser.inner_graph_state.get_top_level_symbol() {
        // We use `""` to represent `null
        parser.inner_graph_state.add_usage(
          "".into(),
          InnerGraphMapUsage::TopLevel(current_symbol.clone()),
        );
      } else {
        parser.inner_graph_state.bailout();
      }
    }
    None
  }
}
