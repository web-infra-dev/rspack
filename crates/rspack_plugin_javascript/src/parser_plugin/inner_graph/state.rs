use std::collections::hash_map::Entry;

use rspack_core::UsedByExports;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::{common::Span, ecma::atoms::Atom};

use super::plugin::TopLevelSymbol;
use crate::{
  parser_plugin::inner_graph::plugin::{
    InnerGraphMapSetValue, InnerGraphMapUsage, InnerGraphMapValue,
  },
  visitors::JavascriptParser,
};

pub type UsageCallback = Box<dyn Fn(&mut JavascriptParser, Option<UsedByExports>)>;

#[derive(Default)]
pub struct InnerGraphState {
  pub(crate) inner_graph: HashMap<Atom, InnerGraphMapValue>,
  pub(crate) usage_callback_map: HashMap<Atom, Vec<UsageCallback>>,
  current_top_level_symbol: Option<Atom>,
  enable: bool,

  pub(crate) statement_with_top_level_symbol: HashMap<Span, TopLevelSymbol>,
  pub(crate) statement_pure_part: HashMap<Span, Span>,
  pub(crate) class_with_top_level_symbol: HashMap<Span, TopLevelSymbol>,
  pub(crate) decl_with_top_level_symbol: HashMap<Span, TopLevelSymbol>,
  pub(crate) pure_declarators: HashSet<Span>,
}

impl InnerGraphState {
  pub fn new() -> Self {
    Self {
      ..Default::default()
    }
  }

  pub fn enable(&mut self) {
    self.enable = true;
  }

  pub fn bailout(&mut self) {
    self.enable = false;
  }

  pub fn is_enabled(&self) -> bool {
    self.enable
  }

  pub fn set_top_level_symbol(&mut self, symbol: Option<Atom>) {
    self.current_top_level_symbol = symbol;
  }

  pub fn get_top_level_symbol(&self) -> Option<Atom> {
    if self.is_enabled() {
      self.current_top_level_symbol.clone()
    } else {
      None
    }
  }

  pub fn add_usage(&mut self, symbol: Atom, usage: InnerGraphMapUsage) {
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
