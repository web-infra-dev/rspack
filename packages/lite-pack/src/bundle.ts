import { ModuleNode } from "./module";
import { Graph, ModuleGraph } from "./module-graph";

class Bundle {
  module_ids: string[] = [];
  size: number =0;
  source_bundles: string[] = [];
  
  constructor(id:string, node: ModuleNode){
    this.module_ids = [id];
  }
}
export class BundleGraph extends Graph<Bundle> {
  
}

export class Bundler{
  bundle_id = 0;
  bundle(graph:ModuleGraph){
    const bundle_roots = new Map();
    const bundle_graph = new BundleGraph();
    const entries = graph.getEntries();
    for(const entry_id of graph.getEntries()){
      const bundle = new Bundle(entry_id, graph.getModuleById(entry_id)!)
      const bundle_id = bundle_graph.addNode((this.bundle_id++)+'',bundle)
      bundle_roots.set(entry_id, [bundle_id,bundle_id])
    }
    graph.traverse(entries,{
      node:(id,node)=>{

      }
    })
  }
}