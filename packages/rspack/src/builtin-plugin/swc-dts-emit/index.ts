import {
	BuiltinPluginName,
	type RawSwcDtsEmitRspackPluginOptions
} from "@rspack/binding";
import type { Compiler } from "../../Compiler";

export interface SwcDtsEmitRspackPluginOptions {
	/**
	 * @default d.ts
	 */
	extension?: string;
}

export class SwcDtsEmitRspackPlugin {
	options: SwcDtsEmitRspackPluginOptions;
	constructor(options?: SwcDtsEmitRspackPluginOptions) {
		this.options = options ?? {};
	}
	apply(compiler: Compiler) {
		compiler.__internal__registerBuiltinPlugin({
			name: BuiltinPluginName.SwcDtsEmitRspackPlugin,
			options: this.normalizeOptions(this.options)
		});
	}
	normalizeOptions(
		options: SwcDtsEmitRspackPluginOptions
	): RawSwcDtsEmitRspackPluginOptions {
		const normalzedOptions: RawSwcDtsEmitRspackPluginOptions = options;
		return normalzedOptions;
	}
}
