use crate::utils::eval::BasicEvaluatedExpression;

pub trait JavascriptParserPlugin {
  fn evaluate_typeof(
    &self,
    expression: &swc_core::ecma::ast::Ident,
    start: u32,
    end: u32,
    unresolved_mark: swc_core::common::SyntaxContext, // remove this after `parser.scope.definitions`
  ) -> Option<BasicEvaluatedExpression>;
}

pub type BoxJavascriptParserPlugin = Box<dyn JavascriptParserPlugin>;
