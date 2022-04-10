import { ModuleGraph } from "./module-graph";


export class Chunk {
  id:string;
  modules: string[];
  graph: ModuleGraph;
  constructor(options: {
    id:string,
    graph: ModuleGraph
  }){
    this.modules = [];
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
    for(const id of this.modules){
      const mod = this.graph.getModuleById(id)!
      const code = mod.generator();
      finalCode += code;
    }
    return finalCode;
  }
}