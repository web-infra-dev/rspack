import Module from 'module';
import { ModuleNode } from './module';
import { ModuleGraph } from './module-graph';
import { Runtime } from './runtime';

export class ChunkGroup {
  name: string;
  chunks: Chunk[];
  runtimeChunk: Chunk | null = null;
  parents: Set<ChunkGroup> = new Set();
  children: Set<ChunkGroup> = new Set();
  constructor(name: string) {
    this.name = name;
    this.chunks = [];
  }
  static create(name: string,chunk: Chunk){
    const chunkgroup = new ChunkGroup(name);
    chunkgroup.pushChunk(chunk);
    return chunkgroup;
  }
  pushChunk(chunk: Chunk) {
    this.chunks.push(chunk);
  }
  setName(name:string){
    this.name = name;
  }
  setRuntimeChunk(chunk: Chunk) {
    this.runtimeChunk = chunk;
  }
}
export type Chunktype = 'entry' | 'runtime' | 'vendor' | 'dynamic'
export class Chunk {
  id: string;
  modules: Set<string> = new Set();
  graph: ModuleGraph;
  groups: Set<ChunkGroup> = new Set();
  entryModule!: string;
  name!: string;
  chunkType:Chunktype;
  constructor(options: { id: string; graph: ModuleGraph, chunkType: Chunktype }) {
    this.id = options.id;
    this.graph = options.graph;
    this.chunkType = options.chunkType;
  }
  /**
   * @todo
   * link common chunk  & shared chunk
   */
  link() {}
  render() {
    const runtime = new Runtime();
    let moduleCode = [];
    for (const modId of this.modules) {
      const mod = this.graph.getNodeById(modId)!;
      const code = mod.generator();
      moduleCode.push(code);
    }
    const entryMoule = this.graph.getNodeById(this.entryModule)!;
    const bootstrap = `rs.require(${JSON.stringify(entryMoule?.fullPath)})`;
    return [this.chunkType === 'entry' && runtime.render(), moduleCode.join(';'),this.chunkType === 'entry' && bootstrap].join(';');
  }
  addGroup(group: ChunkGroup) {
    this.groups.add(group);
  }
  addModule(module: string) {
    this.modules.add(module);
  }
  setEntryModule(mode: string) {
    this.entryModule = mode;
  }
}
