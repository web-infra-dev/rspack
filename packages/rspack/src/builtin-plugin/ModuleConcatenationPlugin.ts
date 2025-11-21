import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import { createBuiltinPlugin, RspackBuiltinPlugin } from "./base";

export class ModuleConcatenationPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.ModuleConcatenationPlugin;
	affectedHooks = "compilation" as const;

	raw(): BuiltinPlugin {
		return createBuiltinPlugin(this.name, undefined);
	}
}
