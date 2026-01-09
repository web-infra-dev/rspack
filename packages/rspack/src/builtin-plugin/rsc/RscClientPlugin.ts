import type binding from "@rspack/binding";
import type { Compiler } from "../..";
import { createBuiltinPlugin, RspackBuiltinPlugin } from "../base";
import { type Coordinator, GET_OR_INIT_BINDING } from "./Coordinator";

export class RscClientPlugin extends RspackBuiltinPlugin {
	name = "RscClientPlugin";
	#coordinator: Coordinator;

	constructor(coordinator: Coordinator) {
		super();
		this.#coordinator = coordinator;
	}
	raw(compiler: Compiler): binding.BuiltinPlugin {
		this.#coordinator.applyClientCompiler(compiler);
		return createBuiltinPlugin(
			this.name,
			// @ts-ignore
			this.#coordinator[GET_OR_INIT_BINDING]()
		);
	}
}
