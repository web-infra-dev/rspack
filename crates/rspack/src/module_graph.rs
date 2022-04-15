use std::collections::HashMap;

use petgraph::graph::NodeIndex;
use smol_str::SmolStr;

use crate::{module::Module, module_graph_container::Rel, types::ResolvedId};
type ModulePetGraph = petgraph::graph::DiGraph<SmolStr, Rel>;

pub struct ModuleGraph {
    pub resolved_entries: Vec<ResolvedId>,
    pub id_to_node_idx: HashMap<SmolStr, NodeIndex>,
    pub relation_graph: ModulePetGraph,
    // pub entry_indexs: Vec<NodeIndex>,
    pub ordered_modules: Vec<NodeIndex>,
    // pub mark_box: Arc<Mutex<MarkBox>>,
    pub module_by_id: HashMap<SmolStr, Box<Module>>,
}

impl ModuleGraph {
    pub fn node_idx_of_enties(&self) -> Vec<NodeIndex> {
        self.resolved_entries
            .iter()
            .map(|rid| *self.id_to_node_idx.get(&rid.id).unwrap())
            .collect()
    }
}
