import {
	BuiltinPluginName,
	type RawSwcDtsEmitRspackPluginOptions
} from "@rspack/binding";
import { Compiler } from "../../Compiler";

export interface SwcDtsEmitRspackPluginOptions {
	rootDir: string;
}

export class SwcDtsEmitRspackPlugin {
	apply(compiler: Compiler) {
		compiler.__internal__registerBuiltinPlugin({
			name: BuiltinPluginName.SwcDtsEmitRspackPlugin,
			options: {}
		});
	}
	normalizeOptions(
		options: SwcDtsEmitRspackPluginOptions
	): RawSwcDtsEmitRspackPluginOptions {
		const normalzedOptions: RawSwcDtsEmitRspackPluginOptions = {
			rootDir: options.rootDir
		};
		return normalzedOptions;
	}
}
