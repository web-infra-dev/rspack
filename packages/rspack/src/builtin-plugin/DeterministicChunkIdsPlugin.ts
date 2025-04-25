import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import type { Compiler } from "../Compiler";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";

export class DeterministicChunkIdsPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.DeterministicChunkIdsPlugin;
	affectedHooks = "compilation" as const;

	raw(compiler: Compiler): BuiltinPlugin {
		return createBuiltinPlugin(this.name, undefined);
	}
}
