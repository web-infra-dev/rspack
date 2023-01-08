use hashbrown::HashMap;
use petgraph::stable_graph::{NodeIndex, StableDiGraph};

use super::visitor::SymbolRef;

pub struct SymbolGraph {
  pub(crate) graph: StableDiGraph<SymbolRef, ()>,
  symbol_to_index: HashMap<SymbolRef, NodeIndex>,
}

impl SymbolGraph {
  pub fn new() -> Self {
    Self {
      graph: StableDiGraph::new(),
      symbol_to_index: HashMap::new(),
    }
  }
  pub fn add_node(&mut self, symbol: SymbolRef) -> NodeIndex {
    if let Some(index) = self.symbol_to_index.get(&symbol) {
      *index
    } else {
      let index = self.graph.add_node(symbol.clone());
      self.symbol_to_index.insert(symbol, index);
      index
    }
  }

  pub fn add_edge(&mut self, from: SymbolRef, to: SymbolRef) {
    let from_index = self.add_node(from);
    let to_index = self.add_node(to);
    self.graph.add_edge(from_index, to_index, ());
  }
}
