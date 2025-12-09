import binding from "@rspack/binding";

import { createBuiltinPlugin, RspackBuiltinPlugin } from "../base";
import { Compiler } from "../..";
import { Coordinator } from "./coordinator";

export class ReactClientPlugin extends RspackBuiltinPlugin {
	name = "ReactClientPlugin";
	coordinator: Coordinator;

	constructor(coordinator: Coordinator) {
		super();
		this.coordinator = coordinator;
	}
	raw(compiler: Compiler): binding.BuiltinPlugin {
		this.coordinator.applyClientCompiler(compiler);
		return createBuiltinPlugin(this.name, this.coordinator.getBinding());
	}
}
