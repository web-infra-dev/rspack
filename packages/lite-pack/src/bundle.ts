import { chdir } from 'process';
import { Chunk, ChunkGroup } from './chunk';
import { Graph } from './graph';
import { ModuleNode } from './module';
import { ModuleGraph } from './module-graph';
import path from 'path';
export class ChunkGraph extends Graph<any, any> {}
/**
 * 三者关系
 * chunk
 * chunkGroup
 * entryPoint: entryPoint是一个特殊的chunkGroup
 * entryPoint chunk: 从属于entryPoint的chunk
 * entry module: 从属于entryPoint chunk的 module
 */
export class Bundler {
  graph: ModuleGraph;
  chunks: Chunk[] = [];
  chunkGroups: ChunkGroup[] = [];
  output: Record<string, string>;
  chunk_id = 0;
  constructor(graph: ModuleGraph) {
    this.graph = graph;
    this.output = {};
  }
  build() {
    //this.generate_chunks()
    this.link();
    console.log('chunks:', this.chunks);
    return this.render();
  }
  link() {
    for (const entry of this.graph.getEntries()) {
      const entryNode = this.graph.getNodeById(entry)!;
      const chunk = new Chunk({
        id: entryNode?.entryKey!,
        graph: this.graph,
        chunkType: 'entry'
      });
      this.chunks.push(chunk);
      chunk.setEntryModule(entry);
      chunk.addModule(entry);
    }

    let chunkQueue = [...this.chunks];
    const visit = (id: string, chunk: Chunk) => {
      const queue: string[] = [];
      queue.push(id);
      while (queue.length > 0) {
        const item = queue.shift()!;
        const children = this.graph.getChildrenById(item);
        for (const child of children) {
          if (child.meta.kind === 'dynamic-import') {
            const dynamicChunk = new Chunk({
              id: path.basename(child.to.replace('.js','')),
              graph: this.graph,
              chunkType: 'dynamic'
            });
            dynamicChunk.addModule(child.to);
            dynamicChunk.setEntryModule(child.to);
            this.chunks.push(dynamicChunk);
            chunkQueue.push(dynamicChunk)
          } else {
            chunk.addModule(child.to);
            queue.push(child.to);
          }
        }
      }
    };
    while (chunkQueue.length > 0) {
      const chunk = chunkQueue.shift()!;
      visit(chunk?.entryModule!, chunk);
    }
  }
  render() {
    for (const chunk of this.chunks) {
      this.output[chunk.id] = chunk.render();
    }
  }
}
