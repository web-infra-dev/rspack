import type { JsModuleGraph } from "@rspack/binding";
import { type Dependency, bindingDependencyFactory } from "./Dependency";
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
		const depBinding = bindingDependencyFactory.getBinding(dependency);
		if (depBinding) {
			const binding = this.#inner.getModule(depBinding);
			return binding ? Module.__from_binding(binding) : null;
		}
		return null;
	}

	getResolvedModule(dependency: Dependency): Module | null {
		const depBinding = bindingDependencyFactory.getBinding(dependency);
		if (depBinding) {
			const binding = this.#inner.getResolvedModule(depBinding);
			return binding ? Module.__from_binding(binding) : null;
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
		return this.#inner
			.getOutgoingConnections(Module.__to_binding(module))
			.map(binding => ModuleGraphConnection.__from_binding(binding));
	}

	getIncomingConnections(module: Module): ModuleGraphConnection[] {
		return this.#inner
			.getIncomingConnections(Module.__to_binding(module))
			.map(binding => ModuleGraphConnection.__from_binding(binding));
	}

	getOutgoingConnectionsInOrder(module: Module): ModuleGraphConnection[] {
		return this.#inner
			.getOutgoingConnectionsInOrder(Module.__to_binding(module))
			.map(binding => ModuleGraphConnection.__from_binding(binding));
	}

	getParentBlockIndex(dependency: Dependency): number {
		const depBinding = bindingDependencyFactory.getBinding(dependency);
		if (depBinding) {
			return this.#inner.getParentBlockIndex(depBinding);
		}
		return -1;
	}

	isAsync(module: Module): boolean {
		return this.#inner.isAsync(Module.__to_binding(module));
	}
}
