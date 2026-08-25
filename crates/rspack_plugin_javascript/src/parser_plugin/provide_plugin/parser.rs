use std::sync::Arc;

use cow_utils::CowUtils;
use itertools::Itertools;
use rspack_core::{BoxDependency, DependencyRange};
use rustc_hash::FxHashSet as HashSet;
use swc_next_ecma_ast::{CallExpression, GetSpan, Span};

use super::{super::JavascriptParserPlugin, ProvideValue, VALUE_DEP_PREFIX};
use crate::{
  Atom,
  dependency::ProvideDependency,
  visitors::{HookMemberExpression, Identifier, JavascriptParser},
};

const SOURCE_DOT: &str = r#"."#;
const MODULE_DOT: &str = r#"_dot_"#;

pub struct ProvideParserPlugin {
  provide: Arc<ProvideValue>,
  names: Arc<HashSet<String>>,
}

impl ProvideParserPlugin {
  pub fn new(provide: Arc<ProvideValue>, names: Arc<HashSet<String>>) -> Self {
    Self { provide, names }
  }

  fn add_provide_dep(&self, name: &str, span: Span, parser: &mut JavascriptParser) -> bool {
    if let Some(requests) = self.provide.get(name) {
      let name_identifier = if name.contains(SOURCE_DOT) {
        format!(
          "__rspack_provide_{}",
          name.cow_replace(SOURCE_DOT, MODULE_DOT)
        )
      } else {
        name.to_string()
      };
      let range = DependencyRange::from(span);
      let loc = parser.to_dependency_location(range);
      let dep = ProvideDependency::new(
        range,
        Atom::from(requests[0].as_str()),
        name_identifier,
        requests[1..]
          .iter()
          .map(|s| Atom::from(s.as_str()))
          .collect_vec(),
        loc,
      );
      parser.add_dependency(BoxDependency::new(dep));

      // add value dependency
      let cache_key = format!("{VALUE_DEP_PREFIX}{name}");
      parser
        .build_info
        .value_dependencies
        .insert(cache_key, requests.join("."));
      return true;
    }
    false
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ProvideParserPlugin {
  fn can_rename(&self, _parser: &mut JavascriptParser<'p>, str: &str) -> Option<bool> {
    self.names.contains(str).then_some(true)
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: CallExpression,
    for_name: &str,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    if self.add_provide_dep(for_name, expr.callee(ast).span(ast), parser) {
      // FIXME: webpack use `walk_expression` here
      parser.walk_arguments(
        expr
          .arguments(ast)
          .iter()
          .map(|id| ast.get_node_in_sub_range(id)),
      );
      return Some(true);
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: HookMemberExpression,
    for_name: &str,
  ) -> Option<bool> {
    self
      .add_provide_dep(for_name, expr.span(parser.ast.ast), parser)
      .then_some(true)
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    ident: &Identifier,
    for_name: &str,
  ) -> Option<bool> {
    self
      .add_provide_dep(for_name, ident.span(), parser)
      .then_some(true)
  }
}
