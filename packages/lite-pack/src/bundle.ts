import { Chunk, ChunkGroup } from "./chunk";
import { Graph } from "./graph";
import { ModuleNode } from "./module";
import { ModuleGraph } from "./module-graph";
class BundleGraph extends Graph<any> {

}
function buildBundleGraph(graph:ModuleGraph){
  const bundleGraph = new BundleGraph();
  for(const node of graph.getNodes()){

  }

}
/**
 * 三者关系
 * chunk
 * chunkGroup
 * entryPoint: entryPoint是一个特殊的chunkGroup
 * entryPoint chunk: 从属于entryPoint的chunk
 * entry module: 从属于entryPoint chunk的 module
 */
export class Bundler{
  #bundle_id = 0;
  graph: ModuleGraph
  chunks: Chunk[] = [];
  chunkGroups: ChunkGroup[] = [];
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
    const chunkGroups = [];
    /**
     * step1: create EntryPoint and chunkGroup
     */
    for(const entry of this.graph.getEntries()){
      const entryNode = this.graph.getNodeById(entry)!;
      const chunk = new Chunk({
        id: entryNode?.entryKey!,
        graph: this.graph,
      });
      const entryPoint = new ChunkGroup(entryNode?.entryKey!);
      entryPoint.pushChunk(chunk);
      chunk.addGroup(entryPoint);
      this.chunks.push(chunk);
      this.chunkGroups.push(entryPoint);

      entryNode.addChunk(chunk);
      chunk.addModule(entry);
      chunk.setEntryModule(entry);
      chunk.name = entryNode.entryKey!;
    }
    this.#buildChunkGraph();
  }
  #buildChunkGraph(){
    const  visit = (chunk:Chunk, startId:string)=>{
      const queue:string[] = []
      queue.push(startId)
      while(queue.length >0){
        const item = queue.shift()!
        const children = this.graph.getChildrenById(item)
        for(const child of children){
          chunk.addModule(child)
          queue.push(child);
        }
      }
    }
    for(const chunkGroup of this.chunkGroups){
      for(const chunk of chunkGroup.chunks){
        visit(chunk,chunk.entryModule);
      }
    }
  }
  render(){
    console.log('chunk:',this.chunks)
    for(const chunk of this.chunks ){
      this.output[chunk.id] = chunk.render()
    }
  }
}