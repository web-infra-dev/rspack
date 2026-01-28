use std::sync::Arc;

use cow_utils::CowUtils;
use itertools::Itertools;
use rspack_core::DependencyRange;
use rustc_hash::FxHashSet as HashSet;
use swc_core::{atoms::Atom, common::Spanned};

use super::{super::JavascriptParserPlugin, ProvideValue, VALUE_DEP_PREFIX};
use crate::{dependency::ProvideDependency, visitors::JavascriptParser};

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

  fn add_provide_dep(
    &self,
    name: &str,
    range: DependencyRange,
    parser: &mut JavascriptParser,
  ) -> bool {
    if let Some(requests) = self.provide.get(name) {
      let name_identifier = if name.contains(SOURCE_DOT) {
        format!(
          "__rspack_provide_{}",
          name.cow_replace(SOURCE_DOT, MODULE_DOT)
        )
      } else {
        name.to_string()
      };
      let dep = ProvideDependency::new(
        range,
        Atom::from(requests[0].as_str()),
        name_identifier,
        requests[1..]
          .iter()
          .map(|s| Atom::from(s.as_str()))
          .collect_vec(),
        Some(&parser.source),
      );
      parser.add_dependency(Box::new(dep));

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

impl JavascriptParserPlugin for ProvideParserPlugin {
  fn can_rename(&self, _parser: &mut JavascriptParser, str: &str) -> Option<bool> {
    self.names.contains(str).then_some(true)
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.add_provide_dep(for_name, expr.callee.span().into(), parser) {
      // FIXME: webpack use `walk_expression` here
      parser.walk_expr_or_spread(&expr.args);
      return Some(true);
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    self
      .add_provide_dep(for_name, expr.span().into(), parser)
      .then_some(true)
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    self
      .add_provide_dep(for_name, ident.span.into(), parser)
      .then_some(true)
  }
}
