import binding, {
  type Dependency,
  type JsModuleGraph,
  type JsModuleGraphSnapshot,
} from '@rspack/binding';
import { ExportsInfo } from './ExportsInfo';
import type { ModuleGraphConnection } from './ModuleGraphConnection';
import type { Module } from './Module';

export default class ModuleGraph {
  /** `edgeStates` value for a connection that is not active. */
  static readonly EDGE_STATE_INACTIVE = binding.EDGE_STATE_INACTIVE;
  /** `edgeStates` value for a connection that is active. */
  static readonly EDGE_STATE_ACTIVE = binding.EDGE_STATE_ACTIVE;
  /**
   * `edgeStates` value corresponding to
   * `ModuleGraphConnection.CIRCULAR_CONNECTION`.
   */
  static readonly EDGE_STATE_CIRCULAR = binding.EDGE_STATE_CIRCULAR;
  /**
   * `edgeStates` value corresponding to
   * `ModuleGraphConnection.TRANSITIVE_ONLY`.
   */
  static readonly EDGE_STATE_TRANSITIVE_ONLY =
    binding.EDGE_STATE_TRANSITIVE_ONLY;
  /**
   * `edgeTargets` value for a connection whose target module is absent. The
   * edge is still listed, so edge `k` of a module stays aligned with
   * `getOutgoingConnectionsInOrder(module)[k]`.
   */
  static readonly NO_MODULE = binding.NO_MODULE;

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

  getUsedExports(
    module: Module,
    runtime: string | string[],
  ): string[] | boolean | null {
    return this.#inner.getUsedExports(module, runtime);
  }

  getProvidedExports(module: Module): true | string[] | null {
    return this.#inner.getProvidedExports(module);
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

  /**
   * Every module and every outgoing connection, described once in compact
   * arrays.
   *
   * Modules are listed in `Compilation.modules` order and the outgoing edges
   * of module index `m` are `edgeOffsets[m]..edgeOffsets[m + 1]`, in
   * `getOutgoingConnectionsInOrder` order. Consumers that walk the whole
   * graph can use this instead of per-module and per-connection getters.
   *
   * The snapshot describes the graph at the moment it is taken and is not
   * updated afterwards; take a new one per compilation.
   */
  getSnapshot(): JsModuleGraphSnapshot {
    return this.#inner.getSnapshot();
  }
}
