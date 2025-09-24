import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import { createBuiltinPlugin, RspackBuiltinPlugin } from "./base";

export class DeterministicModuleIdsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.DeterministicModuleIdsPlugin;
	affectedHooks = "compilation" as const;

	raw(): BuiltinPlugin {
		return createBuiltinPlugin(this.name, undefined);
	}
}
