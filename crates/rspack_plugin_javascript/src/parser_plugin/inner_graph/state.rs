use std::{
  collections::hash_map::Entry,
  sync::atomic::{AtomicUsize, Ordering},
};

use rspack_core::DependencyId;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::{atoms::Atom, common::Span};

static TOP_LEVEL_SYMBOL_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct TopLevelSymbol(usize);

impl TopLevelSymbol {
  pub fn is_global(&self) -> bool {
    self.0 == 0
  }

  pub fn global() -> Self {
    Self(0)
  }

  pub fn new() -> Self {
    let id = TOP_LEVEL_SYMBOL_ID.fetch_add(1, Ordering::Relaxed);
    Self(id)
  }

  pub(crate) fn add_depend_on(self, state: &mut InnerGraphState, depend_on: Atom, span: Span) {
    let symbol = state.symbol_map.get_mut(&self).expect("should have symbol");
    symbol.depend_on_pure.insert((depend_on, span));
  }
}

impl Default for TopLevelSymbol {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone)]
pub(super) struct TopLevelSymbolData {
  pub(super) name: Atom,
  pub(super) depend_on_pure: HashSet<(Atom, Span)>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub(super) enum InnerGraphMapValue {
  Set(HashSet<InnerGraphMapSetValue>),
  True,
  #[default]
  Nil,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(super) enum InnerGraphMapSetValue {
  TopLevel(TopLevelSymbol),
  Str(Atom),
}

impl InnerGraphMapSetValue {
  pub(super) fn to_atom(&self, symbol_map: &HashMap<TopLevelSymbol, TopLevelSymbolData>) -> Atom {
    match self {
      InnerGraphMapSetValue::TopLevel(v) => {
        symbol_map.get(v).expect("should have symbol").name.clone()
      }
      InnerGraphMapSetValue::Str(v) => v.clone(),
    }
  }
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum InnerGraphMapUsage {
  TopLevel(TopLevelSymbol),
  Value(Atom),
  True,
}

impl From<InnerGraphMapUsage> for InnerGraphMapSetValue {
  fn from(val: InnerGraphMapUsage) -> Self {
    match val {
      InnerGraphMapUsage::TopLevel(s) => InnerGraphMapSetValue::TopLevel(s),
      InnerGraphMapUsage::Value(v) => InnerGraphMapSetValue::Str(v),
      InnerGraphMapUsage::True => unreachable!("InnerGraphMapUsage::True cannot be converted"),
    }
  }
}

#[derive(Default)]
pub(crate) struct InnerGraphState {
  pub(super) symbol_map: HashMap<TopLevelSymbol, TopLevelSymbolData>,
  pub(super) usage_map: HashMap<TopLevelSymbol, Vec<InnerGraphUsageOperation>>,
  pub(super) inner_graph: HashMap<TopLevelSymbol, InnerGraphMapValue>,
  current_top_level_symbol: Option<TopLevelSymbol>,
  enable: bool,
  pub(super) statement_with_top_level_symbol: HashMap<Span, TopLevelSymbol>,
  pub(super) statement_pure_part: HashMap<Span, Span>,
  pub(super) class_with_top_level_symbol: HashMap<Span, TopLevelSymbol>,
  pub(super) decl_with_top_level_symbol: HashMap<Span, TopLevelSymbol>,
  pub(super) pure_declarators: HashSet<Span>,
}

impl InnerGraphState {
  pub(crate) fn new() -> Self {
    let mut symbol_map = HashMap::<TopLevelSymbol, TopLevelSymbolData>::default();

    symbol_map.insert(
      TopLevelSymbol::global(),
      TopLevelSymbolData {
        name: Atom::new(""),
        depend_on_pure: Default::default(),
      },
    );
    Self {
      symbol_map,
      ..Default::default()
    }
  }

  pub(super) fn top_level_symbol(&self, name: &TopLevelSymbol) -> &TopLevelSymbolData {
    &self.symbol_map[name]
  }

  pub(crate) fn new_top_level_symbol(&mut self, name: Atom) -> TopLevelSymbol {
    let symbol = TopLevelSymbol::new();
    let data = TopLevelSymbolData {
      name,
      depend_on_pure: Default::default(),
    };
    self.symbol_map.insert(symbol, data);
    symbol
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
      self.current_top_level_symbol
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
        let set_value: InnerGraphMapSetValue = usage.into();
        match self.inner_graph.entry(symbol) {
          Entry::Occupied(mut occ) => {
            let val = occ.get_mut();
            match val {
              InnerGraphMapValue::Set(set) => {
                set.insert(set_value);
              }
              InnerGraphMapValue::True => {}
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

#[derive(Debug, Clone)]
pub(crate) enum InnerGraphUsageOperation {
  PureExpression(DependencyId),
  ESMImportSpecifier(DependencyId),
  URLDependency(DependencyId),
}

impl InnerGraphUsageOperation {
  pub(crate) fn dep_id(&self) -> DependencyId {
    match self {
      InnerGraphUsageOperation::PureExpression(dep_id)
      | InnerGraphUsageOperation::ESMImportSpecifier(dep_id)
      | InnerGraphUsageOperation::URLDependency(dep_id) => *dep_id,
    }
  }
}
