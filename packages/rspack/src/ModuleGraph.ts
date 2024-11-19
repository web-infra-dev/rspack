import type { JsModuleGraph } from "@rspack/binding";
import { Dependency } from "./Dependency";
import { Module } from "./Module";

export default class ModuleGraph {
	static __from_binding(binding: JsModuleGraph) {
		return new ModuleGraph(binding);
	}

	#inner: JsModuleGraph;

	private constructor(binding: JsModuleGraph) {
		this.#inner = binding;
	}

	getModule(dependency: Dependency): Module | null {
		const binding = this.#inner.getModule(Dependency.__to_binding(dependency));
		return binding ? Module.__from_binding(binding) : null;
	}
}
