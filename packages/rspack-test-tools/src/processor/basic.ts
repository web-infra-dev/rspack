import { runBuild } from "../helper";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import { BasicRunner, EsmRunner, WebRunner } from "../runner";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	ITestRunner,
	TCompiler,
	TCompilerOptions,
	TTestConfig
} from "../type";
import fs from "fs";
import path from "path";

export interface IBasicProcessorOptions<
	T extends ECompilerType = ECompilerType.Rspack
> {
	preOptions?: (context: ITestContext) => TCompilerOptions<T>;
	postOptions?: (context: ITestContext, options: TCompilerOptions<T>) => void;
	getCompiler: (
		context: ITestContext
	) => (options: TCompilerOptions<T> | TCompilerOptions<T>[]) => TCompiler<T>;
	getBundle: (
		context: ITestContext,
		options: TCompilerOptions<T>
	) => string[] | string | void;
	getRunner?: (
		env: ITestEnv,
		context: ITestContext,
		options: TCompilerOptions<T>,
		file: string
	) => ITestRunner;
	getCompilerOptions: (context: ITestContext) => TCompilerOptions<T>;
	testConfig: TTestConfig<T>;
	name: string;
}

export class BasicTaskProcessor<T extends ECompilerType = ECompilerType.Rspack>
	implements ITestProcessor
{
	constructor(protected _options: IBasicProcessorOptions<T>) {}

	async config(context: ITestContext) {
		context.options<T>(
			() =>
				typeof this._options.preOptions === "function"
					? this._options.preOptions!(context)
					: {},
			this._options.name
		);
		context.options<T>(
			() => this._options.getCompilerOptions(context),
			this._options.name
		);
		context.options<T>(options => {
			if (typeof this._options.postOptions === "function") {
				this._options.postOptions(context, options);
			}
		}, this._options.name);
	}

	async compiler(context: ITestContext) {
		const factory = this._options.getCompiler(context);
		context.compiler<T>(options => factory(options), this._options.name);
	}

	async build(context: ITestContext) {
		await runBuild<T>(context, this._options.name);
	}

	async run(env: ITestEnv, context: ITestContext) {
		if (this._options.testConfig.noTest) return;
		if (typeof this._options.testConfig.beforeExecute === "function") {
			this._options.testConfig.beforeExecute();
		}
		context.options<T>((options: TCompilerOptions<T>) => {
			let bundles =
				this._options.testConfig.bundlePath ||
				this._options.getBundle(context, options);
			if (typeof bundles === "string") {
				bundles = [bundles];
			}
			if (!bundles || !bundles.length) {
				return;
			}

			for (let bundle of bundles!) {
				const runner = (this._options.getRunner || this.createRunner)(
					env,
					context,
					options,
					bundle
				);
				if (!runner) {
					throw new Error("create test runner failed");
				}
				const result = runner.run(bundle);
				context.result<T>((_compiler, res) => {
					res.results ??= [];
					res.results.push(result);
				}, this._options.name);
			}
		}, this._options.name);

		let results: Promise<unknown>[] = [];
		context.result<T>((_compiler, res) => {
			results = res.results || [];
		}, this._options.name);
		await Promise.all(results);

		if (typeof this._options.testConfig.afterExecute === "function") {
			this._options.testConfig.afterExecute();
		}
	}

	async check(env: ITestEnv, context: ITestContext) {
		if (this._options.testConfig.noTest) return;
		const errors: Array<{ message: string; stack?: string }> = (
			context.errors.get(this._options.name) || []
		).map(e => ({
			message: e.message,
			stack: e.stack
		}));
		const warnings: Array<{ message: string; stack?: string }> = [];
		context.stats<T>((_, stats) => {
			if (stats) {
				fs.writeFileSync(
					path.join(context.getDist(), "stats.txt"),
					stats.toString({
						preset: "verbose",
						colors: false
					}),
					"utf-8"
				);
				const jsonStats = stats.toJson({
					errorDetails: true
				});
				fs.writeFileSync(
					path.join(context.getDist(), "stats.json"),
					JSON.stringify(jsonStats, null, 2),
					"utf-8"
				);
				if (jsonStats.errors) {
					errors.push(...jsonStats.errors);
				}
				if (jsonStats.warnings) {
					warnings.push(...jsonStats.warnings);
				}
			}
		}, this._options.name);
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
		context.clearError(this._options.name);
	}

	async before(context: ITestContext): Promise<void> {}
	async after(context: ITestContext): Promise<void> {}
	async beforeAll(context: ITestContext): Promise<void> {}
	async afterAll(context: ITestContext) {
		let task;
		context.compiler((_, compiler) => {
			if (compiler) {
				task = new Promise(resolve => compiler.close(resolve));
			}
		}, this._options.name);
		return task;
	}

	protected createRunner(
		env: ITestEnv,
		context: ITestContext,
		options: TCompilerOptions<T>,
		file: string
	): ITestRunner | null {
		let runner: ITestRunner | null = null;
		context.stats<T>((_, stats) => {
			const runnerOptions = {
				env,
				stats: stats!,
				name: this._options.name,
				runInNewContext: false,
				testConfig: this._options.testConfig,
				source: context.getSource(),
				dist: context.getDist(),
				compilerOptions: options
			};
			if (options.target === "web" || options.target === "webworker") {
				runner = new WebRunner<T>(runnerOptions);
			} else if (options.experiments?.outputModule) {
				runner = new EsmRunner<T>(runnerOptions);
			} else {
				runner = new BasicRunner<T>(runnerOptions);
			}
		}, this._options.name);
		return runner;
	}
}
