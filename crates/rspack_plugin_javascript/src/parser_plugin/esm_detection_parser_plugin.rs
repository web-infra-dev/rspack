use std::sync::Arc;

use rspack_core::{BuildMetaExportsType, ExportsArgument, ModuleArgument, ModuleType};
use rspack_util::{SpanExt, swc::AstSubRangeExt};
use swc_next_ecma_ast::{
  AwaitExpression, CallExpression, ForOfStatement, GetSpan, Program, Span, StmtData,
  UnaryExpression, VariableDeclaration,
};

use super::{JavascriptParserPlugin, javascript_meta_info_plugin::has_top_level_using_declaration};
use crate::{
  dependency::ESMCompatibilityDependency,
  utils::eval::BasicEvaluatedExpression,
  visitors::{JavascriptParser, create_traceable_error},
};

impl JavascriptParser<'_> {
  fn throw_top_level_await_error(&mut self, msg: String, span: Span) {
    self.add_error(
      create_traceable_error(
        "JavaScript parse error".into(),
        msg,
        self.source.to_string(),
        span.into(),
      )
      .into(),
    );
  }

  fn handle_top_level_await_at(&mut self, await_lo: u32) {
    if self.is_esm {
      self.build_meta.set_has_top_level_await(true);
    } else {
      self.throw_top_level_await_error(
        "Top-level-await is only supported in ECMAScript Modules".into(),
        Span::new(await_lo, await_lo + AWAIT_LEN),
      );
    }
  }
}

#[derive(Default)]
pub struct ESMDetectionParserPlugin;

// nonHarmonyIdentifiers
fn is_non_esm_identifier(name: &str) -> bool {
  name == "exports" || name == "define"
}

// Port from https://github.com/webpack/webpack/blob/main/lib/dependencies/HarmonyDetectionParserPlugin.js
#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ESMDetectionParserPlugin {
  fn program(&self, parser: &mut JavascriptParser<'p>, _program: Program) -> Option<bool> {
    let is_strict_esm = matches!(parser.module_type, ModuleType::JsEsm);
    let ast = parser.ast.ast;
    let has_esm_declaration = ast.nodes(_program.body(ast)).any(|statement| {
      matches!(
        ast.stmt_data(statement),
        StmtData::ImportDeclaration(_)
          | StmtData::ExportNamedDeclaration(_)
          | StmtData::ExportDefaultDeclaration(_)
          | StmtData::ExportAllDeclaration(_)
          | StmtData::TsExportAssignment(_)
          | StmtData::TsNamespaceExportDeclaration(_)
      )
    });
    let is_esm = is_strict_esm || has_esm_declaration;

    if is_esm {
      parser.add_presentational_dependency(Arc::new(ESMCompatibilityDependency::new(
        has_top_level_using_declaration(parser.ast.ast, _program),
      )));
      parser.build_meta.set_esm(true);
      parser
        .build_meta
        .set_exports_type(BuildMetaExportsType::Namespace);
      parser.build_info.strict = true;
      parser.build_info.exports_argument = ExportsArgument::RspackExports;
    }

    if is_strict_esm {
      parser.build_meta.set_strict_esm_module(true);
      parser.build_info.module_argument = ModuleArgument::RspackModule;
    }

    None
  }

  fn top_level_await_expr(&self, parser: &mut JavascriptParser<'p>, expr: AwaitExpression) {
    parser.handle_top_level_await_at(expr.span(parser.ast.ast).real_lo());
  }

  fn top_level_for_of_await_stmt(&self, parser: &mut JavascriptParser<'p>, stmt: ForOfStatement) {
    parser.handle_top_level_await_at(stmt.span(parser.ast.ast).real_lo() + FOR_LEN);
  }

  fn top_level_await_using_declaration(
    &self,
    parser: &mut JavascriptParser<'p>,
    decl: VariableDeclaration,
  ) {
    parser.handle_top_level_await_at(decl.span(parser.ast.ast).real_lo());
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: UnaryExpression,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    (parser.is_esm && is_non_esm_identifier(for_name)).then(|| {
      let span = expr.span(parser.ast.ast);
      BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi())
    })
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: UnaryExpression,
    for_name: &str,
  ) -> Option<bool> {
    (parser.is_esm && is_non_esm_identifier(for_name)).then_some(true)
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    _span: Span,
    for_name: &str,
  ) -> Option<bool> {
    (parser.is_esm && is_non_esm_identifier(for_name)).then_some(true)
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: CallExpression,
    for_name: &str,
  ) -> Option<bool> {
    if parser.is_esm && is_non_esm_identifier(for_name) {
      if for_name == "define" {
        // Track import bindings in callbacks before skipping AMD handling.
        let ast = parser.ast.ast;
        parser.walk_arguments(ast.nodes(expr.arguments(ast)));
      }
      return Some(true);
    }
    None
  }
}

/// "await".len();
const AWAIT_LEN: u32 = 5;
/// "for ".len();
const FOR_LEN: u32 = 4;
