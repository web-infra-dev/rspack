import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import type { Compiler } from "../Compiler";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";

export class MangleExportsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.MangleExportsPlugin;
	affectedHooks = "compilation" as const;

	constructor(private deterministic: boolean) {
		super();
	}

	raw(compiler: Compiler): BuiltinPlugin {
		return createBuiltinPlugin(this.name, this.deterministic);
	}
}
