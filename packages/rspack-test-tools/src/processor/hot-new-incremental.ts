import {
	ECompilerType,
	type ITestContext,
	type TCompilerOptions
} from "../type";
import type { IBasicProcessorOptions } from "./basic";
import { HotProcessor } from "./hot";

export interface IHotNewIncrementalProcessorOptions<T extends ECompilerType>
	extends Omit<IBasicProcessorOptions<T>, "runable"> {
	target: TCompilerOptions<T>["target"];
}

export class HotNewIncrementalProcessor<
	T extends ECompilerType
> extends HotProcessor<T> {
	constructor(protected _hotOptions: IHotNewIncrementalProcessorOptions<T>) {
		super(_hotOptions);
	}

	static defaultOptions<T extends ECompilerType>(
		this: HotNewIncrementalProcessor<T>,
		context: ITestContext
	): TCompilerOptions<T> {
		const options = super.defaultOptions<T>(context);
		if (this._hotOptions.compilerType === ECompilerType.Rspack) {
			const rspackOptions = options as TCompilerOptions<ECompilerType.Rspack>;
			rspackOptions.experiments ??= {};
			rspackOptions.experiments.rspackFuture ??= {};
			rspackOptions.experiments.rspackFuture.newIncremental = true;
		} else {
			throw new Error(
				"HotNewIncrementalProcessor should only used for Rspack."
			);
		}
		return options;
	}
}
