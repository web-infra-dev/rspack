import type { JsModuleGraph } from "@rspack/binding";
import { bindingDependencyFactory, Dependency } from "./Dependency";
import { ExportsInfo } from "./ExportsInfo";
import { Module } from "./Module";
import { ModuleGraphConnection } from "./ModuleGraphConnection";
import { VolatileMap } from "./util/volatile";

export default class ModuleGraph {
	static __from_binding(binding: JsModuleGraph) {
		return new ModuleGraph(binding);
	}

	#inner: JsModuleGraph;
	#resolvedModuleMap = new VolatileMap<Dependency, Module | null>();
	#outgoingConnectionsMap = new VolatileMap<Module, ModuleGraphConnection[]>();
	#parentBlockIndexMap = new VolatileMap<Dependency, number>();
	#isAsyncMap = new VolatileMap<Module, boolean>();

	private constructor(binding: JsModuleGraph) {
		this.#inner = binding;
	}

	getModule(dependency: Dependency): Module | null {
		const depBinding = bindingDependencyFactory.getBinding(dependency);
		if (depBinding) {
			const binding = this.#inner.getModule(depBinding);
			return binding ? Module.__from_binding(binding) : null;
		}
		return null;
	}

	getResolvedModule(dependency: Dependency): Module | null {
		if (this.#resolvedModuleMap.get(dependency) !== undefined) {
			return this.#resolvedModuleMap.get(dependency)!;
		}
		const depBinding = bindingDependencyFactory.getBinding(dependency);
		if (depBinding) {
			const binding = this.#inner.getResolvedModule(depBinding);
			const module = binding ? Module.__from_binding(binding) : null;
			this.#resolvedModuleMap.set(dependency, module);
			return module;
		}
		return null;
	}

	getParentModule(dependency: Dependency): Module | null {
		const depBinding = bindingDependencyFactory.getBinding(dependency);
		if (depBinding) {
			const binding = this.#inner.getParentModule(depBinding);
			return binding ? Module.__from_binding(binding) : null;
		}
		return null;
	}

	getIssuer(module: Module): Module | null {
		const binding = this.#inner.getIssuer(Module.__to_binding(module));
		return binding ? Module.__from_binding(binding) : null;
	}

	getExportsInfo(module: Module): ExportsInfo {
		return ExportsInfo.__from_binding(
			this.#inner.getExportsInfo(Module.__to_binding(module))
		);
	}

	getConnection(dependency: Dependency): ModuleGraphConnection | null {
		const depBinding = bindingDependencyFactory.getBinding(dependency);
		if (depBinding) {
			const binding = this.#inner.getConnection(depBinding);
			return binding ? ModuleGraphConnection.__from_binding(binding) : null;
		}
		return null;
	}

	getOutgoingConnections(module: Module): ModuleGraphConnection[] {
		if (this.#outgoingConnectionsMap.get(module)) {
			return this.#outgoingConnectionsMap.get(module)!;
		}
		const connections = this.#inner
			.getOutgoingConnections(Module.__to_binding(module))
			.map(binding => ModuleGraphConnection.__from_binding(binding));
		this.#outgoingConnectionsMap.set(module, connections);
		return connections;
	}

	getParentBlockIndex(dependency: Dependency): number {
		if (this.#parentBlockIndexMap.get(dependency) !== undefined) {
			return this.#parentBlockIndexMap.get(dependency)!;
		}
		const depBinding = bindingDependencyFactory.getBinding(dependency);
		if (depBinding) {
			const index = this.#inner.getParentBlockIndex(depBinding);
			this.#parentBlockIndexMap.set(dependency, index);
			return index;
		}
		return -1;
	}

	isAsync(module: Module): boolean {
		if (this.#isAsyncMap.get(module) !== undefined) {
			return this.#isAsyncMap.get(module)!;
		}
		const result = this.#inner.isAsync(Module.__to_binding(module));
		this.#isAsyncMap.set(module, result);
		return result;
	}
}
