import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import { BasicRunner, EsmRunner, WebRunner } from "../runner";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	ITestRunner,
	TCompilerFactory,
	TCompilerOptions,
	TTestConfig
} from "../type";
import fs from "fs";
import path from "path";

export interface IBasicProcessorOptions<
	T extends ECompilerType = ECompilerType.Rspack
> {
	defaultOptions?: (context: ITestContext) => TCompilerOptions<T>;
	overrideOptions?: (
		context: ITestContext,
		options: TCompilerOptions<T>
	) => void;
	compilerFactory: (context: ITestContext) => TCompilerFactory<T>;
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
	compilerOptions: (context: ITestContext) => TCompilerOptions<T>;
	testConfig: TTestConfig<T>;
	name: string;
}

export class BasicTaskProcessor<T extends ECompilerType = ECompilerType.Rspack>
	implements ITestProcessor
{
	constructor(protected _options: IBasicProcessorOptions<T>) {}

	async config(context: ITestContext) {
		const compiler = this.getCompiler(context);
		if (typeof this._options.defaultOptions === "function") {
			compiler.setOptions(this._options.defaultOptions(context));
		}

		compiler.mergeOptions(this._options.compilerOptions(context));

		if (typeof this._options.overrideOptions === "function") {
			const compilerOptions = compiler.getOptions();
			this._options.overrideOptions(context, compilerOptions);
		}
	}

	async compiler(context: ITestContext) {
		const compiler = this.getCompiler(context);
		compiler.createCompiler();
	}

	async build(context: ITestContext) {
		const compiler = this.getCompiler(context);
		await compiler.build();
	}

	async run(env: ITestEnv, context: ITestContext) {
		if (this._options.testConfig.noTest) return;
		if (typeof this._options.testConfig.beforeExecute === "function") {
			this._options.testConfig.beforeExecute();
		}
		const compiler = this.getCompiler(context);
		let bundles =
			this._options.testConfig.bundlePath ||
			this._options.getBundle(context, compiler.getOptions());
		if (typeof bundles === "string") {
			bundles = [bundles];
		}
		if (!bundles || !bundles.length) {
			return;
		}

		for (let bundle of bundles!) {
			const runner = (this._options.getRunner || this.createRunner.bind(this))(
				env,
				context,
				compiler.getOptions(),
				bundle
			);
			if (!runner) {
				throw new Error("create test runner failed");
			}
			const mod = runner.run(bundle);
			const result =
				context.getResult<Array<Promise<unknown>>>(this._options.name) || [];
			result.push(mod);
			context.setResult<Array<Promise<unknown>>>(this._options.name, result);
		}

		const results =
			context.getResult<Array<Promise<unknown>>>(this._options.name) || [];
		await Promise.all(results);

		if (typeof this._options.testConfig.afterExecute === "function") {
			this._options.testConfig.afterExecute();
		}
	}

	async check(env: ITestEnv, context: ITestContext) {
		if (this._options.testConfig.noTest) return;
		const errors: Array<{ message: string; stack?: string }> = (
			context.getError(this._options.name) || []
		).map(e => ({
			message: e.message,
			stack: e.stack
		}));
		const warnings: Array<{ message: string; stack?: string }> = [];
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
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
		const compiler = this.getCompiler(context);
		await compiler.close();
	}

	protected createRunner(
		env: ITestEnv,
		context: ITestContext,
		options: TCompilerOptions<T>,
		file: string
	): ITestRunner | null {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
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
			return new WebRunner<T>(runnerOptions);
		} else if (
			path.extname(file) === ".mjs" &&
			options.experiments?.outputModule
		) {
			return new EsmRunner<T>(runnerOptions);
		}
		return new BasicRunner<T>(runnerOptions);
	}

	protected getCompiler(context: ITestContext) {
		return context.getCompiler(
			this._options.name,
			this._options.compilerFactory(context)
		);
	}
}
