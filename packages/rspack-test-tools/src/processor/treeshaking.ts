import { ECompilerType, ITestContext, TCompilerOptions } from "../type";
import { SnapshotProcessor } from "./snapshot";
import { RspackBuiltinProcessor } from "./builtin";

export interface IRspackTreeShakingProcessorOptions {
	name: string;
	snapshot: string;
}

export class RspackTreeShakingProcessor extends SnapshotProcessor<ECompilerType.Rspack> {
	constructor(
		protected _treeShakingOptions: IRspackTreeShakingProcessorOptions
	) {
		super({
			configFiles: ["rspack.config.js", "webpack.config.js"],
			snapshot: _treeShakingOptions.snapshot,
			compilerType: ECompilerType.Rspack,
			defaultOptions: RspackBuiltinProcessor.defaultOptions,
			overrideOptions: RspackTreeShakingProcessor.overrideOptions,
			name: _treeShakingOptions.name,
			runable: false
		});
	}

	static overrideOptions(
		context: ITestContext,
		options: TCompilerOptions<ECompilerType.Rspack>
	) {
		options.target = options.target || ["web", "es2022"];
		options.optimization ??= {};
		options.optimization.providedExports = true;
		options.optimization.innerGraph = true;
		options.optimization.usedExports = true;
	}
}
