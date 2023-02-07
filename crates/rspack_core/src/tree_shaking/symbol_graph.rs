use petgraph::{
  stable_graph::{NodeIndex, StableDiGraph},
  visit::EdgeRef,
};
use rspack_symbol::{IndirectTopLevelSymbol, StarSymbol, Symbol};
use rustc_hash::FxHashMap;

use super::visitor::SymbolRef;
use crate::contextify;

#[derive(Default)]
pub struct SymbolGraph {
  pub(crate) graph: StableDiGraph<SymbolRef, ()>,
  pub(crate) symbol_to_index: FxHashMap<SymbolRef, NodeIndex>,
  pub(crate) node_index_to_symbol: FxHashMap<NodeIndex, SymbolRef>,
}

impl SymbolGraph {
  pub fn add_node(&mut self, symbol: &SymbolRef) -> NodeIndex {
    if let Some(index) = self.symbol_to_index.get(symbol) {
      *index
    } else {
      let index = self.graph.add_node(symbol.clone());
      self.symbol_to_index.insert(symbol.clone(), index);
      self.node_index_to_symbol.insert(index, symbol.clone());
      index
    }
  }

  pub fn has_node(&mut self, symbol: &SymbolRef) -> bool {
    self.symbol_to_index.contains_key(symbol)
  }

  pub fn get_node_index(&self, symbol: &SymbolRef) -> Option<&NodeIndex> {
    self.symbol_to_index.get(symbol)
  }
  pub fn get_symbol(&self, index: &NodeIndex) -> Option<&SymbolRef> {
    self.node_index_to_symbol.get(index)
  }

  // #[track_caller]
  pub fn add_edge(&mut self, from: &SymbolRef, to: &SymbolRef) {
    let from_index = self.add_node(from);
    let to_index = self.add_node(to);
    if !self.graph.contains_edge(from_index, to_index) {
      self.graph.add_edge(from_index, to_index, ());
    }
  }

  pub fn remove_edge(&mut self, from: &SymbolRef, to: &SymbolRef) {
    let from_index = match self.get_node_index(from) {
      Some(index) => *index,
      None => {
        eprintln!("Can't get node index for symbol {from:?}");
        return;
      }
    };
    let to_index = match self.get_node_index(to) {
      Some(index) => *index,
      None => {
        eprintln!("Can't get node index for symbol {to:?}");
        return;
      }
    };
    if let Some(index) = self.graph.find_edge(from_index, to_index) {
      self.graph.remove_edge(index);
    };
  }

  pub fn symbol_refs(&self) -> std::collections::hash_map::Keys<SymbolRef, NodeIndex> {
    self.symbol_to_index.keys()
  }

  pub fn node_indexes(&self) -> std::collections::hash_map::Keys<NodeIndex, SymbolRef> {
    self.node_index_to_symbol.keys()
  }
}

pub fn generate_debug_symbol_graph(g: &SymbolGraph, context: &str) -> StableDiGraph<SymbolRef, ()> {
  let mut debug_graph = SymbolGraph::default();
  for node_index in g.node_indexes() {
    for edge in g.graph.edges(*node_index) {
      let from = edge.source();
      let to = edge.target();
      let from_symbol = simplify_symbol_ref(g.get_symbol(&from).expect(""), context);
      let to_symbol = simplify_symbol_ref(g.get_symbol(&to).expect(""), context);
      debug_graph.add_edge(&from_symbol, &to_symbol);
    }
  }
  debug_graph.graph
}

pub fn simplify_symbol_ref(symbol_ref: &SymbolRef, context: &str) -> SymbolRef {
  match symbol_ref {
    SymbolRef::Direct(direct) => SymbolRef::Direct(Symbol::new(
      contextify(context, direct.uri().as_str()).into(),
      direct.id().clone(),
      *direct.ty(),
    )),
    SymbolRef::Indirect(indirect) => SymbolRef::Indirect(IndirectTopLevelSymbol {
      src: contextify(context, indirect.src.as_str()).into(),
      importer: contextify(context, indirect.importer().as_str()).into(),
      ..indirect.clone()
    }),
    SymbolRef::Star(star) => SymbolRef::Star(StarSymbol {
      src: contextify(context, star.src.as_str()).into(),
      binding: star.binding.clone(),
      module_ident: star.module_ident,
      ty: star.ty,
    }),
  }
}
