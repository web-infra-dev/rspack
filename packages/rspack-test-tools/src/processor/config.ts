import { ECompilerType, ITestContext, TCompilerOptions } from "..";
import { IMultiTaskProcessorOptions, MultiTaskProcessor } from "./base";

export interface IRspackConfigProcessorOptions {
	name: string;
}

export class RspackConfigProcessor extends MultiTaskProcessor<ECompilerType.Rspack> {
	constructor(options: IRspackConfigProcessorOptions) {
		super({
			preOptions: RspackConfigProcessor.preOptions,
			postOptions: RspackConfigProcessor.postOptions,
			getCompiler: () => require("@rspack/core").rspack,
			getBundle: () => ["main.js"],
			configFiles: ["rspack.config.js", "webpack.config.js"],
			name: options.name
		});
	}
	static preOptions(
		context: ITestContext
	): TCompilerOptions<ECompilerType.Rspack> {
		return {
			context: context.getSource(),
			mode: "development",
			target: "node",
			output: {
				path: context.getDist()
			}
		};
	}
	static postOptions(
		options: TCompilerOptions<ECompilerType.Rspack>,
		context: ITestContext
	): void {
		if (!options.entry) {
			options.entry = {
				main: "./"
			};
		}
	}
}
