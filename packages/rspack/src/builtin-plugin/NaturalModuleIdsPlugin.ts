import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import type { Compiler } from "../Compiler";
import { createBuiltinPlugin, RspackBuiltinPlugin } from "./base";

export class NaturalModuleIdsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.NaturalModuleIdsPlugin;
	affectedHooks = "compilation" as const;

	raw(compiler: Compiler): BuiltinPlugin {
		return createBuiltinPlugin(this.name, undefined);
	}
}
