import { runBuild } from "../helper";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import { BasicRunner, EsmRunner, WebRunner } from "../runner";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	TCompiler,
	TCompilerOptions,
	TTestConfig
} from "../type";

export interface IBasicProcessorOptions<
	T extends ECompilerType = ECompilerType.Rspack
> {
	preOptions?: (context: ITestContext) => TCompilerOptions<T>;
	postOptions?: (context: ITestContext, options: TCompilerOptions<T>) => void;
	getCompiler: (
		context: ITestContext
	) => (options: TCompilerOptions<T>) => TCompiler<T>;
	getBundle: (
		context: ITestContext,
		options: TCompilerOptions<T>
	) => string[] | string | void;
	getCompilerOptions: (context: ITestContext) => TCompilerOptions<T>;
	testConfig: TTestConfig<T>;
	name: string;
}

export class BasicTaskProcessor<T extends ECompilerType = ECompilerType.Rspack>
	implements ITestProcessor
{
	constructor(protected options: IBasicProcessorOptions<T>) {}

	async config(context: ITestContext) {
		context.options<T>(
			() =>
				typeof this.options.preOptions === "function"
					? this.options.preOptions!(context)
					: {},
			this.options.name
		);
		context.options<T>(
			() => this.options.getCompilerOptions(context),
			this.options.name
		);
		context.options<T>(options => {
			if (typeof this.options.postOptions === "function") {
				this.options.postOptions(context, options);
			}
		}, this.options.name);
	}

	async compiler(context: ITestContext) {
		const factory = this.options.getCompiler(context);
		context.compiler<T>(options => factory(options), this.options.name);
	}

	async build(context: ITestContext) {
		await runBuild<T>(context, this.options.name);
	}

	async run(env: ITestEnv, context: ITestContext) {
		if (typeof this.options.testConfig.beforeExecute === "function") {
			this.options.testConfig.beforeExecute();
		}
		context.options<T>((options: TCompilerOptions<T>) => {
			let bundles = this.options.getBundle(context, options);
			if (typeof bundles === "string") {
				bundles = [bundles];
			}
			if (!bundles || !bundles.length) {
				return;
			}
			context.stats<T>((_, stats) => {
				const runnerOptions = {
					env,
					stats: stats!,
					name: this.options.name,
					runInNewContext: false,
					testConfig: this.options.testConfig,
					source: context.getSource(),
					dist: context.getDist(),
					compilerOptions: options
				};
				let runner;
				if (options.target === "web" || options.target === "webworker") {
					runner = new WebRunner<T>(runnerOptions);
				} else if (options.experiments?.outputModule) {
					runner = new EsmRunner<T>(runnerOptions);
				} else {
					runner = new BasicRunner<T>(runnerOptions);
				}
				for (let bundle of bundles!) {
					const result = runner.run(bundle);
					context.result<T, Object>(
						_compiler => ({
							exports: result
						}),
						this.options.name
					);
				}
			}, this.options.name);
		}, this.options.name);

		if (typeof this.options.testConfig.afterExecute === "function") {
			this.options.testConfig.afterExecute();
		}
	}

	async check(env: ITestEnv, context: ITestContext) {
		const errors: Array<{ message: string; stack?: string }> = (
			context.errors.get(this.options.name) || []
		).map(e => ({
			message: e.message,
			stack: e.stack
		}));
		const warnings: Array<{ message: string; stack?: string }> = [];
		context.stats<T>((_, stats) => {
			if (stats) {
				const jsonStats = stats.toJson({
					errorDetails: true
				});
				if (jsonStats.errors) {
					errors.push(...jsonStats.errors);
				}
				if (jsonStats.warnings) {
					warnings.push(...jsonStats.warnings);
				}
			}
		}, this.options.name);
		await new Promise<void>((resolve, reject) => {
			checkArrayExpectation(
				context.getSource(),
				{ errors },
				"error",
				"Error",
				reject
			);
			resolve();
		});
		await new Promise<void>((resolve, reject) => {
			checkArrayExpectation(
				context.getSource(),
				{ warnings },
				"warning",
				"Warning",
				reject
			);
			resolve();
		});
		context.clearError(this.options.name);
	}
}
