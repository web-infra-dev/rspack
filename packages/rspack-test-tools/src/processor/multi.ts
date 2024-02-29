import { readConfigFile } from "../helper";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
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
	constructor(protected opts: IMultiTaskProcessorOptions<T>) {
		super({
			getCompiler: opts.getCompiler,
			getBundle: (context, _) => {
				return this.multiCompilerOptions.reduce<string[]>(
					(res, compilerOptions, index) => {
						const curBundles = opts.getBundle(index, context, compilerOptions);
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
				return this.createRunner(
					env,
					context,
					this.multiCompilerOptions[this.files[file]],
					file
				)!;
			},
			getCompilerOptions: () => ({}),
			testConfig: opts.testConfig,
			name: opts.name
		});
	}

	async config(context: ITestContext) {
		this.multiCompilerOptions = [];
		const source = context.getSource();
		const caseOptions: TCompilerOptions<T>[] = Array.isArray(
			this.opts.configFiles
		)
			? readConfigFile(source, this.opts.configFiles!)
			: [{}];

		for (let [index, options] of caseOptions.entries()) {
			const compilerOptions = merge(
				typeof this.opts.preOptions === "function"
					? this.opts.preOptions!(index, context)
					: {},
				options
			);

			if (typeof this.opts.postOptions === "function") {
				this.opts.postOptions!(index, context, compilerOptions);
			}

			this.multiCompilerOptions.push(compilerOptions);
		}
	}

	async compiler(context: ITestContext) {
		const factory = this.options.getCompiler(context);
		context.compiler<T>(
			options => factory(this.multiCompilerOptions),
			this.options.name
		);
	}
}
