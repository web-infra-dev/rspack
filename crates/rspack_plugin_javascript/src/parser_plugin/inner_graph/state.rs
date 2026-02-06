use std::collections::hash_map::Entry;

use rspack_core::DependencyRange;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::common::Span;

use super::plugin::TopLevelSymbol;
use crate::parser_plugin::inner_graph::plugin::{
  InnerGraphMapSetValue, InnerGraphMapUsage, InnerGraphMapValue,
};

/// The operation to be performed when processing inner graph usage.
#[derive(Debug, Clone)]
pub(crate) enum InnerGraphUsageOperation {
  /// Create PureExpressionDependency with the given range
  PureExpression(DependencyRange),
  /// Set used_by_exports on ESMImportSpecifierDependency at the given dependency index
  ESMImportSpecifier(usize),
  /// Set used_by_exports on URLDependency at the given dependency index
  URLDependency(usize),
}

#[derive(Default)]
pub(crate) struct InnerGraphState {
  pub(crate) inner_graph: HashMap<TopLevelSymbol, InnerGraphMapValue>,
  pub(crate) usage_map: HashMap<TopLevelSymbol, Vec<InnerGraphUsageOperation>>,
  current_top_level_symbol: Option<TopLevelSymbol>,
  enable: bool,

  pub(crate) statement_with_top_level_symbol: HashMap<Span, TopLevelSymbol>,
  pub(crate) statement_pure_part: HashMap<Span, Span>,
  pub(crate) class_with_top_level_symbol: HashMap<Span, TopLevelSymbol>,
  pub(crate) decl_with_top_level_symbol: HashMap<Span, TopLevelSymbol>,
  pub(crate) pure_declarators: HashSet<Span>,
}

impl InnerGraphState {
  pub(crate) fn new() -> Self {
    Self {
      ..Default::default()
    }
  }

  pub(crate) fn enable(&mut self) {
    self.enable = true;
  }

  pub(crate) fn bailout(&mut self) {
    self.enable = false;
  }

  pub(crate) fn is_enabled(&self) -> bool {
    self.enable
  }

  pub(crate) fn set_top_level_symbol(&mut self, symbol: Option<TopLevelSymbol>) {
    self.current_top_level_symbol = symbol;
  }

  pub(crate) fn get_top_level_symbol(&self) -> Option<TopLevelSymbol> {
    if self.is_enabled() {
      self.current_top_level_symbol.clone()
    } else {
      None
    }
  }

  pub(crate) fn add_usage(&mut self, symbol: TopLevelSymbol, usage: InnerGraphMapUsage) {
    if !self.is_enabled() {
      return;
    }

    match usage {
      InnerGraphMapUsage::True => {
        self.inner_graph.insert(symbol, InnerGraphMapValue::True);
      }
      InnerGraphMapUsage::Value(_) | InnerGraphMapUsage::TopLevel(_) => {
        // SAFETY: we can make sure that the usage is not a `InnerGraphMapSetValue::True` variant.
        let set_value: InnerGraphMapSetValue = usage.into();
        match self.inner_graph.entry(symbol) {
          Entry::Occupied(mut occ) => {
            let val = occ.get_mut();
            match val {
              InnerGraphMapValue::Set(set) => {
                set.insert(set_value);
              }
              InnerGraphMapValue::True => {
                // do nothing, https://github.com/webpack/webpack/blob/e381884115df2e7b8acd651d3bc2ee6fc35b188e/lib/optimize/InnerGraph.js#L92-L94
              }
              InnerGraphMapValue::Nil => {
                *val = InnerGraphMapValue::Set(HashSet::from_iter([set_value]));
              }
            }
          }
          Entry::Vacant(vac) => {
            vac.insert(InnerGraphMapValue::Set(HashSet::from_iter([set_value])));
          }
        }
      }
    }
  }
}
