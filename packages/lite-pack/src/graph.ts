type Visitor<T,M>  = {
  enter?(id:string,node:T): void;
  edge?(edge:Edge<M>):void;
  leave?(id:string,node:T):void;
}
class Edge<T> {
  from:string;
  to:string;
  meta: T;
  constructor(from:string, to:string, meta: T ){
    this.from = from ;
    this.to = to;
    this.meta = meta;
  }
}
export class Graph<Node,Meta> {
  #nodes: Map<string, Node>;
  #edges: Map<string, Edge<Meta>[]>;
  constructor() {
    this.#nodes = new Map();
    this.#edges = new Map();
  }
  getNodes(){
    return this.#nodes;
  }
  getEdges(){
    return this.#edges;
  }
  getNodeById(id: string) {
    return this.#nodes.get(id);
  }
  getChildrenById(id: string) {
    return this.#edges.get(id) ?? [];
  }
  addNode(id: string, node: Node) {
    this.#nodes.set(id, node);
    return id;
  }
  checkNodeExist(id: string) {
    if (!this.#nodes.get(id) && id !== '') {
      console.log('id:', this.#nodes.keys(), id);
      throw new Error(`${id} not exists`);
    }
  }
  addEdge(from: string, to: string, meta:Meta) {
    this.checkNodeExist(from);
    this.checkNodeExist(to);
    const edges = this.#edges.get(from);
    if (edges) {
      const e = new Edge(from, to,meta);
      edges.push(e);
    } else {
      this.#edges.set(from, [new Edge(from,to,meta)]);
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
  traverse(startId: string | string[], visitor: Visitor<Node,Meta>) {
    if (Array.isArray(startId)) {
      for (const id of startId) {
        this.#dfs(id, visitor);
      }
    } else {
      this.#dfs(startId, visitor);
    }
  }
  getEdge(from:string){

  }
  #dfs(startId: string, visitor: Visitor<Node,Meta>) {
    let visited = new Set();
    const walk = (id: string) => {
      if (visited.has(id)) {
        return;
      }
      const module = this.getNodeById(id);
      if (!module) {
        throw new Error('module not exist:' + id);
      }
      visitor.enter?.(id,module);
      console.log('children:', this.getChildrenById(id))
      for (const child of this.getChildrenById(id)) {
        visitor.edge?.(child);
        walk(child.to);
      }
      visitor.leave?.(id,module);
    };
    return walk(startId);
  }
}

