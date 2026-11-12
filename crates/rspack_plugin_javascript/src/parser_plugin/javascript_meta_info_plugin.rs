use rspack_util::swc::AstSubRangeExt;
use swc_next_ecma_ast::{Ast, CallExpression, DeclData, Program, StmtData, VariableKind};

use super::{
  JavascriptParserPlugin,
  inner_graph::state::{InnerGraphMapUsage, TopLevelSymbol},
};
use crate::visitors::JavascriptParser;

pub(crate) fn has_top_level_using_declaration(ast: &Ast<'_>, program: Program) -> bool {
  ast.nodes(program.body(ast)).any(|statement| {
    let StmtData::Declaration(declaration) = ast.stmt_data(statement) else {
      return false;
    };
    matches!(
      ast.decl_data(declaration),
      DeclData::VariableDeclaration(variable)
        if matches!(variable.kind(ast), VariableKind::Using | VariableKind::AwaitUsing)
    )
  })
}

pub struct JavascriptMetaInfoPlugin;

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for JavascriptMetaInfoPlugin {
  fn program(&self, parser: &mut JavascriptParser<'p>, program: Program) -> Option<bool> {
    if has_top_level_using_declaration(parser.ast.ast, program) {
      // Concatenation would defer disposal to the end of the enclosing scope.
      parser.build_info.module_concatenation_bailout = Some("a top-level using declaration".into());
    }
    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: CallExpression,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "eval" {
      parser.build_info.module_concatenation_bailout = Some("eval()".into());
      parser.build_info.has_direct_eval = true;
      if let Some(top_level_symbol) = parser.inner_graph.get_top_level_symbol() {
        parser.inner_graph.add_usage(
          TopLevelSymbol::global(),
          InnerGraphMapUsage::TopLevel(top_level_symbol),
        );
      } else {
        parser.inner_graph.bailout();
      }
    }

    None
  }

  fn finish(&self, parser: &mut JavascriptParser<'p>) -> Option<bool> {
    if parser.build_info.top_level_declarations.is_none() {
      parser.build_info.top_level_declarations = Some(Default::default());
    }
    let variables = parser.definitions_db.current_variables();
    let has_direct_eval = parser.build_info.has_direct_eval;
    for (name, info) in variables {
      if !parser.definitions_db.expect_get_variable(info).is_free() {
        parser
          .build_info
          .top_level_declarations
          .as_mut()
          .expect("must have value")
          .insert(name.clone());
        // Direct eval can reach any top-level binding through a FREE alias;
        // treat all of them as mutated so value-binding emit bails out.
        if has_direct_eval {
          parser.mark_mutated_binding(name);
        }
      }
    }
    None
  }
}
