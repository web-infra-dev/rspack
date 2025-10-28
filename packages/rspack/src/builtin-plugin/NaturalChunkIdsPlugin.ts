import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import { createBuiltinPlugin, RspackBuiltinPlugin } from "./base";

export class NaturalChunkIdsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.NaturalChunkIdsPlugin;
	affectedHooks = "compilation" as const;

	raw(): BuiltinPlugin {
		return createBuiltinPlugin(this.name, undefined);
	}
}
