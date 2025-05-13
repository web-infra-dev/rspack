import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import type { Compiler } from "../Compiler";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";

export class DeterministicModuleIdsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.DeterministicModuleIdsPlugin;
	affectedHooks = "compilation" as const;

	raw(compiler: Compiler): BuiltinPlugin {
		return createBuiltinPlugin(this.name, undefined);
	}
}
