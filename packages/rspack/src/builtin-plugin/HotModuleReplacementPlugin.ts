import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";

import type { Compiler } from "../Compiler";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";

export class HotModuleReplacementPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.HotModuleReplacementPlugin;

	raw(compiler: Compiler): BuiltinPlugin {
		if (compiler.options.output.strictModuleErrorHandling === undefined) {
			compiler.options.output.strictModuleErrorHandling = true;
		}
		return createBuiltinPlugin(this.name, undefined);
	}
}
