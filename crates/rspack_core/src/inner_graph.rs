// Inner graph state for cross-module pure function analysis
use std::{
  collections::hash_map::Entry,
  sync::atomic::{AtomicUsize, Ordering},
};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::{atoms::Atom, common::Span};

use crate::DependencyId;

static TOP_LEVEL_SYMBOL_ID: AtomicUsize = AtomicUsize::new(1);

/// Represents a top-level symbol in the inner graph
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TopLevelSymbol(usize);

impl TopLevelSymbol {
  pub fn is_global(&self) -> bool {
    self.0 == 0
  }

  pub fn global() -> Self {
    // use 0 to present global symbol
    Self(0)
  }

  pub fn new() -> Self {
    let id = TOP_LEVEL_SYMBOL_ID.fetch_add(1, Ordering::Relaxed);
    Self(id)
  }

  pub fn add_depend_on(self, state: &mut InnerGraphState, depend_on: Atom, span: Span) {
    let symbol = state.symbol_map.get_mut(&self).expect("should have symbol");
    symbol.depend_on_pure.insert((depend_on, span));
  }
}

impl Default for TopLevelSymbol {
  fn default() -> Self {
    Self::new()
  }
}

/// Data associated with a top-level symbol
#[derive(Debug, Clone)]
pub struct TopLevelSymbolData {
  pub name: Atom,
  pub depend_on_pure: HashSet<(Atom, Span)>,
}

/// Value in the inner graph map for a symbol
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum InnerGraphMapValue {
  Set(HashSet<InnerGraphMapSetValue>),
  True,
  #[default]
  Nil,
}

/// Set value variant for inner graph map
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum InnerGraphMapSetValue {
  TopLevel(TopLevelSymbol),
  Str(Atom),
}

impl InnerGraphMapSetValue {
  pub fn to_atom(&self, symbol_map: &HashMap<TopLevelSymbol, TopLevelSymbolData>) -> Atom {
    match self {
      InnerGraphMapSetValue::TopLevel(v) => {
        symbol_map.get(v).expect("should have symbol").name.clone()
      }
      InnerGraphMapSetValue::Str(v) => v.clone(),
    }
  }
}

/// Usage type for adding to inner graph
#[derive(PartialEq, Eq, Debug)]
pub enum InnerGraphMapUsage {
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

/// State for inner graph analysis
#[derive(Default)]
pub struct InnerGraphState {
  pub symbol_map: HashMap<TopLevelSymbol, TopLevelSymbolData>,
  pub usage_map: HashMap<TopLevelSymbol, Vec<InnerGraphUsageOperation>>,
  pub inner_graph: HashMap<TopLevelSymbol, InnerGraphMapValue>,
  pub current_top_level_symbol: Option<TopLevelSymbol>,
  enable: bool,
  pub statement_with_top_level_symbol: HashMap<Span, TopLevelSymbol>,
  pub statement_pure_part: HashMap<Span, Span>,
  pub class_with_top_level_symbol: HashMap<Span, TopLevelSymbol>,
  pub decl_with_top_level_symbol: HashMap<Span, TopLevelSymbol>,
  pub pure_declarators: HashSet<Span>,
}

impl InnerGraphState {
  pub fn new() -> Self {
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

  pub fn top_level_symbol(&self, name: &TopLevelSymbol) -> &TopLevelSymbolData {
    &self.symbol_map[name]
  }

  pub fn top_level_symbol_mut(&mut self, name: &TopLevelSymbol) -> &mut TopLevelSymbolData {
    self
      .symbol_map
      .get_mut(name)
      .expect("should have symbol in map")
  }

  pub fn new_top_level_symbol(&mut self, name: Atom) -> TopLevelSymbol {
    let symbol = TopLevelSymbol::new();
    let data = TopLevelSymbolData {
      name,
      depend_on_pure: Default::default(),
    };
    self.symbol_map.insert(symbol, data);
    symbol
  }

  pub fn add_depend_on(&mut self, symbol: &TopLevelSymbol, name: Atom, span: Span) {
    let data = self.top_level_symbol_mut(symbol);
    data.depend_on_pure.insert((name, span));
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

  pub fn set_top_level_symbol(&mut self, symbol: Option<TopLevelSymbol>) {
    self.current_top_level_symbol = symbol;
  }

  pub fn get_top_level_symbol(&self) -> Option<TopLevelSymbol> {
    if self.is_enabled() {
      self.current_top_level_symbol
    } else {
      None
    }
  }

  pub fn add_usage(&mut self, symbol: TopLevelSymbol, usage: InnerGraphMapUsage) {
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
              InnerGraphMapValue::True => {
                // do nothing
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

impl std::fmt::Debug for InnerGraphState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("InnerGraphState")
      .field("symbol_map", &self.symbol_map)
      .field("inner_graph", &self.inner_graph)
      .field("current_top_level_symbol", &self.current_top_level_symbol)
      .field("enable", &self.enable)
      .field(
        "statement_with_top_level_symbol",
        &self.statement_with_top_level_symbol,
      )
      .field("statement_pure_part", &self.statement_pure_part)
      .field(
        "class_with_top_level_symbol",
        &self.class_with_top_level_symbol,
      )
      .field(
        "decl_with_top_level_symbol",
        &self.decl_with_top_level_symbol,
      )
      .field("pure_declarators", &self.pure_declarators)
      .finish()
  }
}

/// The operation to be performed when processing inner graph usage.
#[derive(Debug, Clone)]
pub enum InnerGraphUsageOperation {
  /// Create PureExpressionDependency with the given range
  PureExpression(DependencyId),
  /// Set used_by_exports on ESMImportSpecifierDependency at the given dependency index
  ESMImportSpecifier(DependencyId),
  /// Set used_by_exports on URLDependency at the given dependency index
  URLDependency(DependencyId),
}
