import type {
  Dependency,
  JsModuleGraph,
  ModuleGraphConnection,
} from '@rspack/binding';
import { ExportsInfo } from './ExportsInfo';
import type { Module } from './Module';

export default class ModuleGraph {
  static __from_binding(binding: JsModuleGraph) {
    return new ModuleGraph(binding);
  }

  #inner: JsModuleGraph;

  constructor(binding: JsModuleGraph) {
    this.#inner = binding;
  }

  getModule(dependency: Dependency): Module | null {
    return this.#inner.getModule(dependency);
  }

  getResolvedModule(dependency: Dependency): Module | null {
    return this.#inner.getResolvedModule(dependency);
  }

  getParentModule(dependency: Dependency): Module | null {
    return this.#inner.getParentModule(dependency);
  }

  getIssuer(module: Module): Module | null {
    return this.#inner.getIssuer(module);
  }

  getExportsInfo(module: Module): ExportsInfo {
    return ExportsInfo.__from_binding(this.#inner.getExportsInfo(module));
  }

  getConnection(dependency: Dependency): ModuleGraphConnection | null {
    return this.#inner.getConnection(dependency);
  }

  getOutgoingConnections(module: Module): ModuleGraphConnection[] {
    return this.#inner.getOutgoingConnections(module);
  }

  getIncomingConnections(module: Module): ModuleGraphConnection[] {
    return this.#inner.getIncomingConnections(module);
  }

  getParentBlockIndex(dependency: Dependency): number {
    return this.#inner.getParentBlockIndex(dependency);
  }

  isAsync(module: Module): boolean {
    return this.#inner.isAsync(module);
  }

  getOutgoingConnectionsInOrder(module: Module): ModuleGraphConnection[] {
    return this.#inner.getOutgoingConnectionsInOrder(module);
  }
}
