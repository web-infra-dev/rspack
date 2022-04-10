type Visitor<T>  = {
  enter?(id:string,node:T): void;
  edge?(from:string,to:string):void;
  leave?(id:string,node:T):void;
}
export class Graph<T> {
  #nodes: Map<string, T>;
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
  addNode(id: string, node: T) {
    this.#nodes.set(id, node);
    return id;
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
      this.#edges.set(from, [to]);
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
  traverse(startId: string | string[], visitor: Visitor<T>) {
    if (Array.isArray(startId)) {
      for (const id of startId) {
        this.#dfs(id, visitor);
      }
    } else {
      this.#dfs(startId, visitor);
    }
  }
  #dfs(startId: string, visitor: Visitor<T>) {
    let visited = new Set();
    const walk = (id: string) => {
      if (visited.has(id)) {
        return;
      }
      const module = this.getModuleById(id);
      if (!module) {
        throw new Error('module not exist:' + id);
      }
      visitor.enter?.(id,module);
      console.log('children:', this.getChildrenById(id))
      for (const child of this.getChildrenById(id)) {

        visitor.edge?.(id, child);
        walk(child);
      }
      visitor.leave?.(id,module);
    };
    return walk(startId);
  }
}

