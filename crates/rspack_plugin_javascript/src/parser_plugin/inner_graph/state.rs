use std::hash::{Hash, Hasher};

use rspack_util::atom::AtomKey;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_next_ecma_ast::Span;

use crate::Atom;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct TopLevelSymbol(u32);

impl TopLevelSymbol {
  pub(super) fn from_index(index: usize) -> Self {
    Self(u32::try_from(index).expect("too many inner graph symbols"))
  }

  pub(super) fn index(self) -> usize {
    self.0 as usize
  }

  pub fn is_global(&self) -> bool {
    self.0 == 0
  }

  pub fn global() -> Self {
    Self(0)
  }

  pub(crate) fn add_depend_on(self, state: &mut InnerGraphState, depend_on: Atom, span: Span) {
    let symbol = &mut state.symbols[self.index()].data;
    symbol.depend_on_pure.insert((depend_on, span));
  }
}

impl Default for TopLevelSymbol {
  fn default() -> Self {
    Self::global()
  }
}

#[derive(Debug, Clone)]
pub(super) struct TopLevelSymbolData {
  pub(super) name: Atom,
  pub(super) depend_on_pure: HashSet<(Atom, Span)>,
}

pub(super) struct TopLevelSymbolState {
  pub(super) data: TopLevelSymbolData,
  pub(super) usages: Vec<InnerGraphUsageOperation>,
  pub(super) graph: Option<InnerGraphMapValue>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub(super) enum InnerGraphMapValue {
  Set(HashSet<InnerGraphMapSetValue>),
  True,
  #[default]
  Nil,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub(super) enum InnerGraphMapSetValue {
  TopLevel(TopLevelSymbol),
  Str(Atom),
}

impl Hash for InnerGraphMapSetValue {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
      Self::TopLevel(value) => {
        0u8.hash(state);
        value.hash(state);
      }
      Self::Str(value) => {
        1u8.hash(state);
        AtomKey::from_atom_ref(value).hash(state);
      }
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
  pub(super) symbols: Vec<TopLevelSymbolState>,
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
      symbols: vec![TopLevelSymbolState {
        data: TopLevelSymbolData {
          name: Atom::new(""),
          depend_on_pure: Default::default(),
        },
        usages: Vec::new(),
        graph: None,
      }],
      ..Default::default()
    }
  }

  pub(super) fn symbol_count(&self) -> usize {
    self.symbols.len()
  }

  pub(super) fn symbols(&self) -> impl Iterator<Item = (TopLevelSymbol, &TopLevelSymbolData)> {
    self
      .symbols
      .iter()
      .enumerate()
      .map(|(index, state)| (TopLevelSymbol::from_index(index), &state.data))
  }

  pub(super) fn graph_symbols(&self) -> impl Iterator<Item = TopLevelSymbol> + '_ {
    self
      .symbols
      .iter()
      .enumerate()
      .filter(|(_, state)| state.graph.is_some())
      .map(|(index, _)| TopLevelSymbol::from_index(index))
  }

  pub(super) fn graph(&self, symbol: TopLevelSymbol) -> Option<&InnerGraphMapValue> {
    self.symbols[symbol.index()].graph.as_ref()
  }

  pub(super) fn graph_mut(&mut self, symbol: TopLevelSymbol) -> Option<&mut InnerGraphMapValue> {
    self.symbols[symbol.index()].graph.as_mut()
  }

  pub(super) fn set_graph(&mut self, symbol: TopLevelSymbol, graph: InnerGraphMapValue) {
    self.symbols[symbol.index()].graph = Some(graph);
  }

  pub(super) fn take_graph(&mut self, symbol: TopLevelSymbol) -> Option<InnerGraphMapValue> {
    self.symbols[symbol.index()].graph.take()
  }

  pub(super) fn has_usage_operations(&self) -> bool {
    self.symbols.iter().any(|state| !state.usages.is_empty())
  }

  pub(super) fn add_usage_operation(
    &mut self,
    symbol: TopLevelSymbol,
    operation: InnerGraphUsageOperation,
  ) {
    self.symbols[symbol.index()].usages.push(operation);
  }

  pub(super) fn take_usage_operations(
    &mut self,
    symbol: TopLevelSymbol,
  ) -> Vec<InnerGraphUsageOperation> {
    std::mem::take(&mut self.symbols[symbol.index()].usages)
  }

  pub(super) fn top_level_symbol(&self, name: &TopLevelSymbol) -> &TopLevelSymbolData {
    &self.symbols[name.index()].data
  }

  pub(crate) fn new_top_level_symbol(&mut self, name: Atom) -> TopLevelSymbol {
    let symbol = TopLevelSymbol::from_index(self.symbols.len());
    self.symbols.push(TopLevelSymbolState {
      data: TopLevelSymbolData {
        name,
        depend_on_pure: Default::default(),
      },
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

    match usage {
      InnerGraphMapUsage::True => {
        self.set_graph(symbol, InnerGraphMapValue::True);
      }
      InnerGraphMapUsage::Value(_) | InnerGraphMapUsage::TopLevel(_) => {
        let set_value: InnerGraphMapSetValue = usage.into();
        match self.graph_mut(symbol) {
          Some(InnerGraphMapValue::Set(set)) => {
            set.insert(set_value);
          }
          Some(InnerGraphMapValue::True) => {}
          Some(value @ InnerGraphMapValue::Nil) => {
            *value = InnerGraphMapValue::Set(HashSet::from_iter([set_value]));
          }
          None => self.set_graph(
            symbol,
            InnerGraphMapValue::Set(HashSet::from_iter([set_value])),
          ),
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
