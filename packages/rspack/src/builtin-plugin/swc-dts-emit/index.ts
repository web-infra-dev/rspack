import { BuiltinPluginName } from "@rspack/binding";
import { Compiler } from "../../Compiler";

export interface SwcDtsEmitRspackPluginOptions {}

export class SwcDtsEmitRspackPlugin {
	apply(compiler: Compiler) {
		compiler.__internal__registerBuiltinPlugin({
			name: BuiltinPluginName.SwcDtsEmitRspackPlugin,
			options: {}
		});
	}
}
