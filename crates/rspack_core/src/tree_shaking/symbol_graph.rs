use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use rspack_symbol::{IndirectTopLevelSymbol, IndirectType};
use rustc_hash::FxHashMap;

use super::{debug_care_module_id, visitor::SymbolRef};

pub struct SymbolGraph {
  pub(crate) graph: StableDiGraph<SymbolRef, ()>,
  pub(crate) symbol_to_index: FxHashMap<SymbolRef, NodeIndex>,
  pub(crate) node_index_to_symbol: FxHashMap<NodeIndex, SymbolRef>,
}

// #[track_caller]
// fn prints_calling_location() {
//   let caller_location = std::panic::Location::caller();
//   let caller_line_number = caller_location.line();
//   println!("called from line: {}", caller_line_number);
// }
impl SymbolGraph {
  pub fn new() -> Self {
    Self {
      graph: StableDiGraph::new(),
      symbol_to_index: FxHashMap::default(),
      node_index_to_symbol: FxHashMap::default(),
    }
  }

  pub fn add_node(&mut self, symbol: &SymbolRef) -> NodeIndex {
    // if debug_care_module_id(symbol.module_identifier().as_str()) {
    //   dbg!(&symbol);
    // }
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
    // let to_is_valid = match to {
    //   SymbolRef::Indirect(IndirectTopLevelSymbol {
    //     ty: IndirectType::Import(local, _),
    //     ..
    //   }) => local == "track",
    //   _ => false,
    // };
    // let from_is_valid = match from {
    //   SymbolRef::Indirect(IndirectTopLevelSymbol {
    //     ty: IndirectType::Import(local, _),
    //     ..
    //   }) => local == "a",
    //   _ => false,
    // };

    // if from_is_valid && to_is_valid {
    //   let caller_location = std::panic::Location::caller();
    //   let caller_line_number = caller_location.line();
    //   println!("called from line: {}, ", caller_line_number,);
    // }
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
        eprintln!("Can't get node index for symbol {:?}", from);
        return;
      }
    };
    let to_index = match self.get_node_index(to) {
      Some(index) => *index,
      None => {
        eprintln!("Can't get node index for symbol {:?}", to);
        return;
      }
    };
    match self.graph.find_edge(from_index, to_index) {
      Some(index) => {
        self.graph.remove_edge(index);
      }
      None => {}
    };
  }

  pub fn symbol_refs(&self) -> std::collections::hash_map::Keys<SymbolRef, NodeIndex> {
    self.symbol_to_index.keys()
  }

  pub fn node_indexes(&self) -> std::collections::hash_map::Keys<NodeIndex, SymbolRef> {
    self.node_index_to_symbol.keys()
  }
}
