import Module from "module";
import { ModuleNode } from "./module";
import { ModuleGraph } from "./module-graph";

export class ChunkGroup {
  name:string;
  chunks: Chunk[];
  runtimeChunk: Chunk|null = null;
  constructor(name:string){
    this.name = name;
    this.chunks = [];
  }
  pushChunk(chunk:Chunk){
    this.chunks.push(chunk);
  }
  setRuntimeChunk(chunk:Chunk){
    this.runtimeChunk = chunk;
  }
}
export class Chunk {
  id:string;
  modules: Set<ModuleNode> = new Set();
  graph: ModuleGraph;
  groups: Set<ChunkGroup>= new Set();
  entryModule!: ModuleNode;
  name!:string;
  constructor(options: {
    id:string,
    graph: ModuleGraph,
  }){
    this.id = options.id;
    this.graph = options.graph;
  }
  /**
   * @todo
   * link common chunk  & shared chunk
   */
  link(){

  }
  render(){
    let finalCode = '';
    for(const mod of this.modules){
      const code = mod.generator();
      finalCode += code;
    }
    return finalCode;
  }
  addGroup(group:ChunkGroup){
    this.groups.add(group);
  }
  addModule(module:ModuleNode){
    this.modules.add(module);
  }
  setEntryModule(mode:ModuleNode){
    this.entryModule = mode
  }
}