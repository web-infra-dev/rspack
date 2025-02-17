import type { JsModuleGraph } from "@rspack/binding";
import { Dependency } from "./Dependency";
import { ExportsInfo } from "./ExportsInfo";
import { Module } from "./Module";
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
		const binding = this.#inner.getModule(Dependency.__to_binding(dependency));
		return binding ? Module.__from_binding(binding) : null;
	}

	getResolvedModule(dependency: Dependency): Module | null {
		const binding = this.#inner.getResolvedModule(
			Dependency.__to_binding(dependency)
		);
		return binding ? Module.__from_binding(binding) : null;
	}

	getParentModule(dependency: Dependency): Module | null {
		const binding = this.#inner.getParentModule(
			Dependency.__to_binding(dependency)
		);
		return binding ? Module.__from_binding(binding) : null;
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
		const binding = this.#inner.getConnection(
			Dependency.__to_binding(dependency)
		);
		return binding ? ModuleGraphConnection.__from_binding(binding) : null;
	}

	getOutgoingConnections(module: Module): ModuleGraphConnection[] {
		return this.#inner
			.getOutgoingConnections(Module.__to_binding(module))
			.map(binding => ModuleGraphConnection.__from_binding(binding));
	}

	getIncomingConnections(module: Module): ModuleGraphConnection[] {
		return this.#inner
			.getIncomingConnections(Module.__to_binding(module))
			.map(binding => ModuleGraphConnection.__from_binding(binding));
	}

	getParentBlockIndex(dependency: Dependency): number {
		return this.#inner.getParentBlockIndex(Dependency.__to_binding(dependency));
	}

	isAsync(module: Module): boolean {
		return this.#inner.isAsync(Module.__to_binding(module));
	}
}
