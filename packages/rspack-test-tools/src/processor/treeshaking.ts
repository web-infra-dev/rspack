import { ECompilerType, ITestContext, TCompilerOptions } from "../type";
import { BuiltinProcessor } from "./builtin";
import { ISnapshotProcessorOptions, SnapshotProcessor } from "./snapshot";

export interface ITreeShakingProcessorOptions<T extends ECompilerType>
	extends Omit<ISnapshotProcessorOptions<T>, "runable"> { }

export class TreeShakingProcessor<
	T extends ECompilerType
> extends SnapshotProcessor<T> {
	constructor(protected _treeShakingOptions: ITreeShakingProcessorOptions<T>) {
		super({
			defaultOptions: BuiltinProcessor.defaultOptions,
			overrideOptions: TreeShakingProcessor.overrideOptions<T>,
			runable: false,
			..._treeShakingOptions
		});
	}

	static overrideOptions<T extends ECompilerType>(
		context: ITestContext,
		options: TCompilerOptions<T>
	) {
		options.target = options.target || ["web", "es2022"];
		options.optimization ??= {};
		options.optimization.providedExports = true;
		options.optimization.innerGraph = true;
		options.optimization.usedExports = true;
	}
}
