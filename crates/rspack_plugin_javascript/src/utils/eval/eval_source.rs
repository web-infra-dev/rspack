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

      // ParsedJavaScriptAst represents programs in the main scan path. Build a
      // minimal valid program root around the fragment so any evaluation hook
      // consulting parser.ast observes a self-consistent AST.
      let program = {
        // SAFETY: All handles belong to this fragment. Adding its program root
        // only appends storage, preserving existing nodes and handles.
        let ast = unsafe { ast.ast_mut() };
        let expression_span = expression.span(ast);
        let statement = ast.expression_statement(expression_span, expression);
        let directives = ast.add_sub_range(std::iter::empty::<Directive>());
        let body = ast.add_sub_range([Stmt::ExpressionStatement(statement)]);
        let program = ast.program(expression_span, directives, body, None);
        ast.set_root_program(program);
        program
      };

      let SemanticReturn {
        semantic,
        diagnostics: _,
      } = analyze(ast, AnalyzeOptions::default());
      let (ast, semantic) = semantic.into_parts();
      let ast = &*allocator.alloc(ast);
      // SAFETY: Fragment evaluation retains its owning AST and never mutates it.
      let ast = unsafe { ast.ast() };
      let comments = &*allocator.alloc(RspackComments::from_ast(ast));
      let semantic = &*allocator.alloc(semantic);
      let fragment_ast = &*allocator.alloc(ParsedJavaScriptAst {
        ast,
        comments,
        semantic,
        program,
      });

      // Flat-AST expressions are handles rather than self-contained nodes.
      // Evaluate while the parser tracks the owning fragment, matching the
      // legacy owned-expression behavior without rewriting nested owners.
      Some(parser.evaluate_expression_in_ast(expression, fragment_ast))
    }
  }
}
