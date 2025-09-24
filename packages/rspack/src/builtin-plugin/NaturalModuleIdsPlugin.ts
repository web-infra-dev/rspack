import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import { createBuiltinPlugin, RspackBuiltinPlugin } from "./base";

export class NaturalModuleIdsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.NaturalModuleIdsPlugin;
	affectedHooks = "compilation" as const;

	raw(): BuiltinPlugin {
		return createBuiltinPlugin(this.name, undefined);
	}
}
