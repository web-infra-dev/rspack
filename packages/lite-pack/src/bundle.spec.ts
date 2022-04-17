import { build } from "esbuild";
import { ChunkGraph } from "./bundle"
import { Chunk } from "./chunk";
import { ModuleNode } from "./module";
import { ModuleGraph } from "./module-graph";

function buildNode(path:string, isEntry=false){
  return new ModuleNode({
    path,
    isEntry,
    resolveDir: '',
    importer: '',
    importKind: 'import-statement',
    compiler: null as any
  })
}
function buildChunkNode(id:string){
  return new Chunk({
    id,
    graph: null as any,
    chunkType: 'dynamic'
  })
}
function setupModuleGraph(){
  const moduleGraph = new ModuleGraph();
  const m1 = buildNode('a', true)
  const m2 = buildNode('b');
  moduleGraph.addNode(m1.path, m1);
  moduleGraph.addNode(m2.path, m2);
  moduleGraph.addEdge(m1.path, m2.path, {
    kind: 'import-statement'
  });
  return moduleGraph;
}
function buildChunkGraph(m: ModuleGraph){
  const c1 = buildChunkNode('c1');
  const c2 = buildChunkNode('c2');
  const chunkGraph = new ChunkGraph();
}
describe('chunk-graph', () => {
  it('chunk', () => {
    const moduleGraph = setupModuleGraph();
  })
})

