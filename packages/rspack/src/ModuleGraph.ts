import type { JsModuleGraph } from "@rspack/binding";
import { bindingDependencyFactory, Dependency } from "./Dependency";
import { ExportsInfo } from "./ExportsInfo";
import { Module } from "./Module";
import { ModuleGraphConnection } from "./ModuleGraphConnection";

class VolatileCache<K, V> {
	#map = new Map<K, V>();

	get(key: K): V | undefined {
		return this.#map.get(key);
	}

	set(key: K, value: V) {
		if (this.#map.size === 0) {
			queueMicrotask(() => {
				this.#map.clear();
			});
		}
		this.#map.set(key, value);
	}

	has(key: K): boolean {
		return this.#map.has(key);
	}
}

export default class ModuleGraph {
	static __from_binding(binding: JsModuleGraph) {
		return new ModuleGraph(binding);
	}

	#inner: JsModuleGraph;
	#resolvedModuleMappings = new VolatileCache<Dependency, Module | null>();
	#outgoingConnectionsMappings = new VolatileCache<Module, ModuleGraphConnection[]>();
	#parentBlockIndexMappings = new VolatileCache<Dependency, number>();
	#isAsyncCache = new VolatileCache<Module, boolean>();

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
		if (this.#resolvedModuleMappings.get(dependency)) {
			return this.#resolvedModuleMappings.get(dependency)!;
		}
		const depBinding = bindingDependencyFactory.getBinding(dependency);
		if (depBinding) {
			const binding = this.#inner.getResolvedModule(depBinding);
			const module = binding ? Module.__from_binding(binding) : null;
			this.#resolvedModuleMappings.set(dependency, module);
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
		if (this.#outgoingConnectionsMappings.get(module)) {
			return this.#outgoingConnectionsMappings.get(module)!;
		}
		const connections = this.#inner
			.getOutgoingConnections(Module.__to_binding(module))
			.map(binding => ModuleGraphConnection.__from_binding(binding));
		this.#outgoingConnectionsMappings.set(module, connections);
		return connections;
	}

	getParentBlockIndex(dependency: Dependency): number {
		if (this.#parentBlockIndexMappings.get(dependency)) {
			return this.#parentBlockIndexMappings.get(dependency)!;
		}
		const depBinding = bindingDependencyFactory.getBinding(dependency);
		if (depBinding) {
			const index = this.#inner.getParentBlockIndex(depBinding);
			this.#parentBlockIndexMappings.set(dependency, index);
			return index;
		}
		return -1;
	}

	isAsync(module: Module): boolean {
		if (this.#isAsyncCache.get(module)) {
			return this.#isAsyncCache.get(module)!;
		}
		const result = this.#inner.isAsync(Module.__to_binding(module));
		this.#isAsyncCache.set(module, result);
		return result;
	}
}
