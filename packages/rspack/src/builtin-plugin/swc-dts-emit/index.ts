import {
	BuiltinPluginName,
	type RawSwcDtsEmitRspackPluginOptions
} from "@rspack/binding";
import type { Compiler } from "../../Compiler";

export interface SwcDtsEmitRspackPluginOptions {
	rootDir: string;
}

export class SwcDtsEmitRspackPlugin {
	options: SwcDtsEmitRspackPluginOptions;
	constructor(options: SwcDtsEmitRspackPluginOptions) {
		this.options = options;
	}
	apply(compiler: Compiler) {
		compiler.__internal__registerBuiltinPlugin({
			name: BuiltinPluginName.SwcDtsEmitRspackPlugin,
			options: this.options
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
