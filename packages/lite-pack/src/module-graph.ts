import { Graph } from './graph';
import { ModuleNode } from './module';

export class ModuleGraph extends Graph<ModuleNode> {
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
