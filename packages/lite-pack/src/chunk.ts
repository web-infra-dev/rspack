import Module from "module";
import { ModuleNode } from "./module";
import { ModuleGraph } from "./module-graph";
import { Runtime } from "./runtime";

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
  modules: Set<string> = new Set();
  graph: ModuleGraph;
  groups: Set<ChunkGroup>= new Set();
  entryModule!: string;
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
    const runtime = new Runtime();
    let moduleCode =[];
    for(const modId of this.modules){
      const mod = this.graph.getModuleById(modId)!;
      const code = mod.generator();
      moduleCode.push(code)
    }
    const entryMoule = this.graph.getModuleById(this.entryModule)!
    const bootstrap = `rs.require(${JSON.stringify(entryMoule?.fullPath)})`
    return [runtime.render(),moduleCode.join(';'),bootstrap].join(';')
  }
  addGroup(group:ChunkGroup){
    this.groups.add(group);
  }
  addModule(module:string){
    this.modules.add(module);
  }
  setEntryModule(mode:string){
    this.entryModule = mode
  }
}