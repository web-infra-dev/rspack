import type { Dependency, JsModuleGraph } from "@rspack/binding";
import { ExportsInfo } from "./ExportsInfo";
import type { Module } from "./Module";
import { ModuleGraphConnection } from "./ModuleGraphConnection";

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
		const binding = this.#inner.getConnection(dependency);
		return binding ? ModuleGraphConnection.__from_binding(binding) : null;
	}

	getOutgoingConnections(module: Module): ModuleGraphConnection[] {
		return this.#inner
			.getOutgoingConnections(module)
			.map(binding => ModuleGraphConnection.__from_binding(binding));
	}

	getIncomingConnections(module: Module): ModuleGraphConnection[] {
		return this.#inner
			.getIncomingConnections(module)
			.map(binding => ModuleGraphConnection.__from_binding(binding));
	}

	getParentBlockIndex(dependency: Dependency): number {
		return this.#inner.getParentBlockIndex(dependency);
	}

	isAsync(module: Module): boolean {
		return this.#inner.isAsync(module);
	}

	getOutgoingConnectionsInOrder(module: Module): ModuleGraphConnection[] {
		return this.#inner
			.getOutgoingConnectionsInOrder(module)
			.map(binding => ModuleGraphConnection.__from_binding(binding));
	}
}
