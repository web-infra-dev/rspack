use rspack_util::atom::Atom;
use rustc_hash::FxHashSet;
use swc_experimental_ecma_ast::{AssignExpr, CallExpr, Ident, MemberExpr, Span};

use super::{
  JavascriptParserPlugin,
  inner_graph::state::{InnerGraphMapUsage, TopLevelSymbol},
};
use crate::visitors::JavascriptParser;

pub struct JavascriptMetaInfoPlugin;

fn bailout_on_object_prototype(parser: &mut JavascriptParser, for_name: &str, members: &[Atom]) {
  let references_object_prototype = for_name == "Object.prototype"
    || (for_name == "Object"
      && members
        .first()
        .is_some_and(|member| member.as_str() == "prototype"));
  if !parser.is_esm && references_object_prototype {
    parser
      .build_info
      .module_concatenation_bailout
      .get_or_insert_with(|| "Object.prototype access in CommonJS".into());
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for JavascriptMetaInfoPlugin {
  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    _identifier: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "arguments" && parser.is_top_level_this() {
      parser
        .build_info
        .module_concatenation_bailout
        .get_or_insert_with(|| "CommonJS arguments".into());
    }

    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: &CallExpr<'_>,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "eval" {
      parser.build_info.module_concatenation_bailout = Some("eval()".into());
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

  fn member(
    &self,
    parser: &mut JavascriptParser<'p>,
    _member_expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    bailout_on_object_prototype(parser, for_name, &[]);
    None
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    _member_expr: &MemberExpr,
    for_name: &str,
    members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    bailout_on_object_prototype(parser, for_name, members);
    None
  }

  fn assign_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    _assign_expr: &AssignExpr,
    members: &[Atom],
    _member_ranges: &[Span],
    for_name: &str,
  ) -> Option<bool> {
    bailout_on_object_prototype(parser, for_name, members);
    None
  }

  fn finish(&self, parser: &mut JavascriptParser<'p>) -> Option<bool> {
    if parser.build_info.top_level_declarations.is_none() {
      parser.build_info.top_level_declarations = Some(FxHashSet::default());
    }
    let variables: Vec<_> = parser
      .get_all_variables_from_current_scope()
      .map(|(name, _)| Atom::new(name))
      .collect();
    for name in variables {
      if parser.is_variable_defined(&name) {
        parser
          .build_info
          .top_level_declarations
          .as_mut()
          .expect("must have value")
          .insert(name);
      }
    }
    None
  }
}
