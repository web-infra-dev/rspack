import { readConfigFile } from "../helper";
import {
	ECompilerType,
	ITestContext,
	ITestProcessor,
	ITestRunner,
	TCompiler,
	TCompilerOptions,
	TTestConfig
} from "../type";
import { BasicTaskProcessor } from "./basic";
import { merge } from "webpack-merge";

export interface IMultiTaskProcessorOptions<
	T extends ECompilerType = ECompilerType.Rspack
> {
	preOptions?: (index: number, context: ITestContext) => TCompilerOptions<T>;
	postOptions?: (
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	) => void;
	getCompiler: (
		context: ITestContext
	) => (options: TCompilerOptions<T> | TCompilerOptions<T>[]) => TCompiler<T>;
	getBundle: (
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	) => string[] | string | void;
	testConfig: TTestConfig<T>;
	name: string;
	configFiles?: string[];
}

export class MultiTaskProcessor<T extends ECompilerType = ECompilerType.Rspack>
	extends BasicTaskProcessor<T>
	implements ITestProcessor
{
	protected multiCompilerOptions: TCompilerOptions<T>[] = [];
	protected files: Record<string, number> = {};
	protected runners: ITestRunner[] = [];
	constructor(protected _multiOptions: IMultiTaskProcessorOptions<T>) {
		super({
			getCompiler: _multiOptions.getCompiler,
			getBundle: (context, _) => {
				return this.multiCompilerOptions.reduce<string[]>(
					(res, compilerOptions, index) => {
						const curBundles = _multiOptions.getBundle(
							index,
							context,
							compilerOptions
						);
						const bundles = Array.isArray(curBundles)
							? curBundles
							: curBundles
							? [curBundles]
							: [];
						for (const bundle of bundles) {
							this.files[bundle] = index;
						}
						return [
							...res,
							...(Array.isArray(bundles) ? bundles : bundles ? [bundles] : [])
						];
					},
					[]
				);
			},
			getRunner: (env, context, options, file) => {
				const index = this.files[file];
				this.runners[index] =
					this.runners[index] ||
					this.createRunner(
						env,
						context,
						this.multiCompilerOptions[this.files[file]],
						file
					)!;
				return this.runners[index];
			},
			getCompilerOptions: () => ({}),
			testConfig: _multiOptions.testConfig,
			name: _multiOptions.name
		});
	}

	async config(context: ITestContext) {
		this.multiCompilerOptions = [];
		const source = context.getSource();
		const caseOptions: TCompilerOptions<T>[] = Array.isArray(
			this._multiOptions.configFiles
		)
			? readConfigFile(source, this._multiOptions.configFiles!)
			: [{}];

		for (let [index, options] of caseOptions.entries()) {
			const compilerOptions = merge(
				typeof this._multiOptions.preOptions === "function"
					? this._multiOptions.preOptions!(index, context)
					: {},
				options
			);

			if (typeof this._multiOptions.postOptions === "function") {
				this._multiOptions.postOptions!(index, context, compilerOptions);
			}

			this.multiCompilerOptions.push(compilerOptions);
		}
	}

	async compiler(context: ITestContext) {
		const factory = this._options.getCompiler(context);
		context.compiler<T>(
			options => factory(this.multiCompilerOptions),
			this._options.name
		);
	}
}
