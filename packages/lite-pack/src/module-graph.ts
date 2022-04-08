import { ModuleNode } from './module';

export class ModuleGraph {
  #nodes: Map<string, ModuleNode>;
  #edges: Map<string, string[]>;
  constructor() {
    this.#nodes = new Map();
    this.#edges = new Map();
  }
  getModuleById(id: string) {
    return this.#nodes.get(id);
  }
  getChildrenById(id: string) {
    return this.#edges.get(id) ?? [];
  }
  addNode(id: string, node: ModuleNode) {
    this.#nodes.set(id, node);
  }
  checkNodeExist(id: string) {
    if (!this.#nodes.get(id) && id !== '') {
      console.log('id:', this.#nodes.keys(), id);
      throw new Error(`${id} not exists`);
    }
  }
  addEdge(from: string, to: string) {
    this.checkNodeExist(from);
    this.checkNodeExist(to);
    const edges = this.#edges.get(from);
    if (edges) {
      edges.push(to);
    } else {
      this.#edges.set(from, []);
    }
  }
  removeEdge(from: string, to: string) {
    this.checkNodeExist(from);
    this.checkNodeExist(to);
    this.#edges.delete(from);
  }
  toJSON() {
    console.log('edges', this.#edges.entries());
    console.log('nodes:', this.#nodes.entries());
  }
  traverse(startId: string, visitor: (module: ModuleNode) => void) {
    this.#dfs(startId, visitor);
  }
  #dfs(startId: string, visitor: (module: ModuleNode) => void) {
    let visited = new Set();
    const walk = (id: string) => {
      if (visited.has(id)) {
        return;
      }

      const module = this.getModuleById(id);
      if (!module) {
        throw new Error('module not exist:' + id);
      }
      visitor(module);
      for (const child of this.getChildrenById(id)) {
        walk(child);
      }
    };
    return walk(startId);
  }
}
