import { ECompilerType, ITestContext, TCompilerOptions } from "../type";
import { RspackBuiltinProcessor } from "./builtin";
import { SnapshotProcessor } from "./snapshot";

export interface IRspackTreeShakingProcessorOptions {
	name: string;
	snapshot: string;
	type: "new" | "builtin";
}

export class RspackTreeShakingProcessor extends SnapshotProcessor<ECompilerType.Rspack> {
	constructor(
		protected _treeShakingOptions: IRspackTreeShakingProcessorOptions
	) {
		super({
			snapshot: _treeShakingOptions.snapshot,
			compilerType: ECompilerType.Rspack,
			defaultOptions: RspackBuiltinProcessor.defaultOptions,
			overrideOptions: RspackTreeShakingProcessor.overrideOptions(
				_treeShakingOptions.type
			),
			name: _treeShakingOptions.name,
			runable: false
		});
	}

	static overrideOptions(type: IRspackTreeShakingProcessorOptions["type"]) {
		return (
			context: ITestContext,
			options: TCompilerOptions<ECompilerType.Rspack>
		) => {
			options.target = options.target || ["web", "es2022"];
			if (type === "new") {
				options.optimization ??= {};
				options.optimization.providedExports = true;
				options.optimization.innerGraph = true;
				options.optimization.usedExports = true;

				options.builtins ??= {};
				options.builtins.treeShaking = false;
			} else {
				options.experiments ??= {};
				options.experiments.rspackFuture ??= {};
				options.experiments.rspackFuture.newTreeshaking = false;
			}
		};
	}
}
