import { chdir } from 'process';
import { Chunk, ChunkGroup } from './chunk';
import { Graph } from './graph';
import { ModuleNode } from './module';
import { ModuleGraph } from './module-graph';
import path from 'path';
import { BundlerOptions } from './compiler';
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
  options: BundlerOptions;
  constructor(graph: ModuleGraph, options: BundlerOptions) {
    this.graph = graph;
    this.output = {};
    this.options = options;
  }
  build() {
    //this.generate_chunks()
    this.link();
    console.log('chunks:', this.chunks);
    return this.render();
  }
  link() {
    console.log('manualChunk:', this.options.manualChunks );
    for (const entry of this.graph.getEntries()) {
      const entryNode = this.graph.getNodeById(entry)!;
      const chunk = new Chunk({
        id: entryNode?.entryKey!,
        graph: this.graph,
        chunkType: 'entry'
      });
      const chunkGroup = ChunkGroup.create(entryNode?.entryKey!,chunk)
      this.chunks.push(chunk);
      this.chunkGroups.push(chunkGroup);
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
            const id = path.basename(child.to.replace('.js',''))
            const dynamicChunk = new Chunk({
              id,
              graph: this.graph,
              chunkType: 'dynamic'
            });
            const chunkGroup = ChunkGroup.create(id, dynamicChunk);
            this.chunkGroups.push(chunkGroup);
            
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
