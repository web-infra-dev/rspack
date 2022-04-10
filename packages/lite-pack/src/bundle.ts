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
  #bundle_id = 0;
  graph: ModuleGraph
  constructor(graph:ModuleGraph){
    this.graph = graph;
  }
  build(){
    this.generate_chunks();
  }
  generate_chunks(){
    const chunk_roots = new Map();
    const bundle_graph = new BundleGraph();
    const graph = this.graph;
    const entries = graph.getEntries();
    for(const entry_id of graph.getEntries()){
      const bundle = new Bundle(entry_id, graph.getModuleById(entry_id)!)
      const chunk_id = bundle_graph.addNode((this.#bundle_id++)+'',bundle)
      chunk_roots.set(entry_id, [chunk_id,chunk_id])
    }
    console.log('chunk_roots:',chunk_roots)
    graph.traverse(entries,{
      enter:(id,node)=>{
        const chunk = chunk_roots.get(id);
        console.log('ch:',chunk)
      },
      edge(from,to){
      },
      leave:(id,node)=>{
      }
    })

    console.log('graph:', bundle_graph)
  }
}