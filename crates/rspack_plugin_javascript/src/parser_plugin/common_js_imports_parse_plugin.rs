use super::JavascriptParserPlugin;
use crate::utils::eval::{self, BasicEvaluatedExpression};

pub struct CommonJsImportsParserPlugin;

impl JavascriptParserPlugin for CommonJsImportsParserPlugin {
  fn evaluate_typeof(
    &self,
    expression: &swc_core::ecma::ast::Ident,
    start: u32,
    end: u32,
    unresolved_mark: swc_core::common::SyntaxContext,
  ) -> Option<BasicEvaluatedExpression> {
    if expression.sym.as_str() == "require" && expression.span.ctxt == unresolved_mark {
      Some(eval::evaluate_to_string("function".to_string(), start, end))
    } else {
      None
    }
  }
}
