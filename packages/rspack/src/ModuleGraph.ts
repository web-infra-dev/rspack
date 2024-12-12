import type { JsModuleGraph } from "@rspack/binding";
import { Dependency } from "./Dependency";
import { ExportsInfo } from "./ExportsInfo";
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

	getIssuer(module: Module): Module | null {
		const binding = this.#inner.getIssuer(Module.__to_binding(module));
		return binding ? Module.__from_binding(binding) : null;
	}

	getExportsInfo(module: Module): ExportsInfo {
		return ExportsInfo.__from_binding(
			this.#inner.getExportsInfo(Module.__to_binding(module))
		);
	}
}
