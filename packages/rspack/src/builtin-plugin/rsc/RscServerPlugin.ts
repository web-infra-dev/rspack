import binding from "@rspack/binding";

import { createBuiltinPlugin, RspackBuiltinPlugin } from "../base";
import { Compiler } from "../..";
import { Coordinator, GET_OR_INIT_BINDING } from "./Coordinator";

export class RscServerPlugin extends RspackBuiltinPlugin {
	name = "RscServerPlugin";
	#coordinator: Coordinator;

	constructor(coordinator: Coordinator) {
		super();
		this.#coordinator = coordinator;
	}

	#resolve(serverCompiler: Compiler) {
		this.#coordinator.applyServerCompiler(serverCompiler);
		// @ts-ignore
		return this.#coordinator[GET_OR_INIT_BINDING]();
	}

	raw(compiler: Compiler): binding.BuiltinPlugin {
		const bindingOptions = this.#resolve(compiler);
		return createBuiltinPlugin(this.name, bindingOptions);
	}
}
