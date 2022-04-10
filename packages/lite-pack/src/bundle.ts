import { Chunk } from "./chunk";
import { ModuleNode } from "./module";
import { ModuleGraph } from "./module-graph";

export class Bundler{
  #bundle_id = 0;
  graph: ModuleGraph
  chunks: Chunk[] = [];
  output: Record<string,string>;
  constructor(graph:ModuleGraph){
    this.graph = graph;
    this.output = {};
  }
  build(){
    this.generate_chunks();
    return this.render();
  }
  generate_chunks(){
    for(const entry of this.graph.getEntries()){
      const entryNode = this.graph.getModuleById(entry);
      const chunk = new Chunk({
        id: entryNode?.entryKey!,
        graph: this.graph
      });
      this.chunks.push(chunk);
    }
  }
  render(){
    console.log('chunk:',this.chunks)
    for(const chunk of this.chunks ){
      this.output[chunk.id] = chunk.render()
    }
  }
}