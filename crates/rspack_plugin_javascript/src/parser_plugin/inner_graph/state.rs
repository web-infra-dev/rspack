use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_next_ecma_ast::Span;

use crate::Atom;

/// A module-local index allocated by InnerGraph, independent of semantic symbol IDs.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct TopLevelSymbol(u32);

impl TopLevelSymbol {
  /// Allocates an index in this InnerGraph's append-only symbol vector.
  pub(super) fn from_index(index: usize) -> Self {
    Self(u32::try_from(index).expect("too many inner graph symbols"))
  }

  /// Addresses the symbol's data and mutable usage state.
  pub(super) fn index(self) -> usize {
    self.0 as usize
  }

  pub fn global() -> Self {
    Self(0)
  }

  pub(crate) fn add_depend_on(self, state: &mut InnerGraphState, depend_on: Atom, span: Span) {
    let symbol = &mut state.symbols[self.index()];
    symbol.depend_on_pure.insert((depend_on, span));
  }
}

#[derive(Debug, Clone)]
pub(super) struct TopLevelSymbolData {
  pub(super) name: Atom,
  pub(super) depend_on_pure: HashSet<(Atom, Span)>,
  /// Dependency operations whose usage is determined by this symbol.
  pub(super) usages: Vec<InnerGraphUsageOperation>,
  /// None distinguishes a missing graph entry from an explicitly unused symbol.
  pub(super) graph: Option<InnerGraphMapValue>,
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
  /// Dense symbol data and usage state; slot zero is the global symbol.
  pub(super) symbols: Vec<TopLevelSymbolData>,
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
    Self {
      symbols: vec![TopLevelSymbolData {
        name: Atom::new(""),
        depend_on_pure: Default::default(),
        usages: Vec::new(),
        graph: None,
      }],
      ..Default::default()
    }
  }

  pub(super) fn top_level_symbol(&self, name: &TopLevelSymbol) -> &TopLevelSymbolData {
    &self.symbols[name.index()]
  }

  pub(crate) fn new_top_level_symbol(&mut self, name: Atom) -> TopLevelSymbol {
    let symbol = TopLevelSymbol::from_index(self.symbols.len());
    self.symbols.push(TopLevelSymbolData {
      name,
      depend_on_pure: Default::default(),
      usages: Vec::new(),
      graph: None,
    });
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

    let graph = &mut self.symbols[symbol.index()].graph;
    match usage {
      InnerGraphMapUsage::True => {
        *graph = Some(InnerGraphMapValue::True);
      }
      InnerGraphMapUsage::Value(_) | InnerGraphMapUsage::TopLevel(_) => {
        let set_value: InnerGraphMapSetValue = usage.into();
        match graph {
          Some(InnerGraphMapValue::Set(set)) => {
            set.insert(set_value);
          }
          Some(InnerGraphMapValue::True) => {}
          None | Some(InnerGraphMapValue::Nil) => {
            *graph = Some(InnerGraphMapValue::Set(HashSet::from_iter([set_value])));
          }
        }
      }
    }
  }
}

#[derive(Debug, Clone)]
pub(crate) enum InnerGraphUsageOperation {
  PureExpression(usize),
  ESMImportSpecifier(usize),
  URLDependency(usize),
}
