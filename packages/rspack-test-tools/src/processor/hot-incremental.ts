import {
	ECompilerType,
	EDocumentType,
	type ITestContext,
	type ITestEnv,
	type TCompilerOptions
} from "../type";
import type { IBasicProcessorOptions } from "./basic";
import { HotProcessor } from "./hot";

export interface IHotIncrementalProcessorOptions<T extends ECompilerType>
	extends Omit<IBasicProcessorOptions<T>, "runable"> {
	target: TCompilerOptions<T>["target"];
	webpackCases: boolean;
}

export class HotIncrementalProcessor<
	T extends ECompilerType
> extends HotProcessor<T> {
	constructor(protected _hotOptions: IHotIncrementalProcessorOptions<T>) {
		super({
			defaultOptions: HotIncrementalProcessor.defaultOptions,
			..._hotOptions
		});
	}

	async run(env: ITestEnv, context: ITestContext) {
		context.setValue(
			this._options.name,
			"documentType",
			this._hotOptions.webpackCases ? EDocumentType.Fake : EDocumentType.JSDOM
		);
		await super.run(env, context);
	}

	async afterAll(context: ITestContext) {
		try {
			await super.afterAll(context);
		} catch (e: any) {
			const isFake =
				context.getValue(this._options.name, "documentType") ===
				EDocumentType.Fake;
			if (isFake && /Should run all hot steps/.test(e.message)) return;
			throw e;
		}
	}

	static defaultOptions<T extends ECompilerType>(
		this: HotIncrementalProcessor<T>,
		context: ITestContext
	): TCompilerOptions<T> {
		const options = super.defaultOptions<T>(context);
		if (this._hotOptions.compilerType === ECompilerType.Rspack) {
			const rspackOptions = options as TCompilerOptions<ECompilerType.Rspack>;
			rspackOptions.experiments ??= {};
			rspackOptions.experiments.incremental ??= "advance-silent";
		} else {
			throw new Error("HotIncrementalProcessor should only used for Rspack.");
		}
		return options;
	}
}
