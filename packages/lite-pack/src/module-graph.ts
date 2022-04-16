import { ImportKind } from 'esbuild';
import { Graph } from './graph';
import { ModuleNode } from './module';
type depMeta = {
  kind: ImportKind
}
export class ModuleGraph extends Graph<ModuleNode,depMeta> {
  #entries: string[] = [];
  constructor() {
    super();
    this.#entries = [];
  }
  getEntries() {
    return this.#entries;
  }

  override addNode(id: string, node: ModuleNode): string {
    super.addNode(id, node);
    if (node.isEntry) {
      this.#entries.push(id);
    }
    return id;
  }
}

export function splitModuleToEntryGroups(graph: ModuleGraph){
  const module_groups = [];
  const queue = [];
  queue.push(...graph.getEntries())
  while(queue.length >0){
    const item = queue.shift();
    
  }

}