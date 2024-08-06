import { __module_graph_inner_is_async } from "@rspack/binding";
import type { Compilation } from "./Compilation";
import type { Module } from "./Module";

export class ModuleGraph {
	constructor(private compilation: Compilation) {}

	isAsync(module: Module) {
		return (
			__module_graph_inner_is_async(
				module.identifier(),
				this.compilation.__internal_getInner()
			) ?? false
		);
	}
}
