use rspack_error::{Error, Severity};
use rspack_util::swc::RspackComments;
use serde_json::json;
use swc_next_ecma_ast::{Directive, GetSpan, Stmt};
use swc_next_ecma_parser::{FragmentContext, NoTokenParserConfig, Options, Parser};
use swc_next_ecma_semantic::{AnalyzeOptions, SemanticReturn, analyze};

use super::BasicEvaluatedExpression;
use crate::visitors::{JavascriptParser, ParsedJavaScriptAst};

#[inline]
pub fn eval_source<'parser>(
  parser: &mut JavascriptParser<'parser>,
  source: String,
  error_title: String,
) -> Option<BasicEvaluatedExpression<'parser>> {
  // Fragment ASTs use the module AST's arena so their source, nodes, semantic
  // model, and wrapper all live for the complete parser lifetime. This lets us
  // reuse the normal evaluator and parser hooks without unsafe owned handles.
  let allocator = parser.ast.ast.allocator();
  let source_in_allocator = allocator.alloc_str(&source);
  let result = Parser::init(
    allocator,
    source_in_allocator,
    Options {
      preserve_parens: false,
      ..Options::default()
    },
    NoTokenParserConfig,
  )
  .parse_expression_fragment(FragmentContext::TopLevel);
  match result {
    Err(diagnostics) => {
      let span = diagnostics
        .first()
        .map(|diagnostic| diagnostic.span)
        .unwrap_or_default();
      let mut error = Error::from_string(
        Some(source.clone()),
        span.start as usize,
        span.end as usize,
        format!("{error_title} warning"),
        format!("failed to parse {}", json!(source.as_str())),
      );
      error.severity = Severity::Warning;
      parser.add_warning(error.into());
      None
    }
    Ok(mut ast) => {
      let expression = ast.root_expression();
      let expression_span = expression.span(&ast);

      // ParsedJavaScriptAst represents programs in the main scan path. Build a
      // minimal valid program root around the fragment so any evaluation hook
      // consulting parser.ast observes a self-consistent AST.
      let statement = ast.expression_statement(expression_span, expression);
      let directives = ast.add_typed_sub_range(std::iter::empty::<Directive>());
      let body = ast.add_typed_sub_range([Stmt::ExpressionStatement(statement)]);
      let program = ast.program(expression_span, directives, body, None);
      ast.set_root_program(program);

      let ast = &*allocator.alloc(ast);
      let comments = &*allocator.alloc(RspackComments::from_ast(ast));
      let SemanticReturn {
        semantic,
        diagnostics: _,
      } = analyze(ast, AnalyzeOptions::default());
      let semantic = &*allocator.alloc(semantic);
      let fragment_ast = &*allocator.alloc(ParsedJavaScriptAst {
        ast,
        comments,
        semantic,
        program,
      });

      let module_ast = std::mem::replace(&mut parser.ast, fragment_ast);
      let mut evaluated = parser.evaluate_expression(expression);
      parser.ast = module_ast;

      // The expression handle indexes the fragment AST and must not escape
      // after the parser switches back to the module AST.
      evaluated.set_expression(None);
      Some(evaluated)
    }
  }
}
