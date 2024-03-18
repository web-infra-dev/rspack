import { readConfigFile } from "../helper";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	ITestRunner,
	TCompilerOptions
} from "../type";
import fs from "fs";
import path from "path";

export interface IBasicProcessorOptions<
	T extends ECompilerType = ECompilerType.Rspack
> {
	defaultOptions?: (context: ITestContext) => TCompilerOptions<T>;
	configFiles?: string[];
	overrideOptions?: (
		context: ITestContext,
		options: TCompilerOptions<T>
	) => void;
	findBundle?: (
		context: ITestContext,
		options: TCompilerOptions<T>
	) => string[] | string | void;
	compilerType: T;
	runable: boolean;
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

		if (Array.isArray(this._options.configFiles)) {
			const fileOptions = readConfigFile<T>(
				this._options.configFiles.map(i => context.getSource(i))
			)[0];
			compiler.mergeOptions(fileOptions);
		}

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
		if (!this._options.runable) return;

		const testConfig = context.getTestConfig();
		if (testConfig.noTest) return;

		if (typeof testConfig.beforeExecute === "function") {
			testConfig.beforeExecute();
		}
		const compiler = this.getCompiler(context);
		let bundles: string[] | void | string;
		if (testConfig.bundlePath) {
			bundles = testConfig.bundlePath;
		} else if (typeof this._options.findBundle === "function") {
			bundles = this._options.findBundle(context, compiler.getOptions());
		} else {
			bundles = [];
		}

		if (typeof bundles === "string") {
			bundles = [bundles];
		}
		if (!bundles || !bundles.length) {
			return;
		}

		for (let bundle of bundles!) {
			const runnerFactory = context.getRunnerFactory(this._options.name);
			if (!runnerFactory) {
				throw new Error(`Test case ${this._options.name} is not runable`);
			}
			const runner = runnerFactory.create(bundle, compiler.getOptions(), env);
			const mod = runner.run(bundle);
			const result =
				context.getValue<Array<Promise<unknown>>>(
					this._options.name,
					"modules"
				) || [];
			result.push(mod);
			context.setValue<Array<Promise<unknown>>>(
				this._options.name,
				"modules",
				result
			);
		}

		const results =
			context.getValue<Array<Promise<unknown>>>(
				this._options.name,
				"modules"
			) || [];
		await Promise.all(results);

		if (typeof testConfig.afterExecute === "function") {
			testConfig.afterExecute();
		}
	}

	async check(env: ITestEnv, context: ITestContext) {
		const testConfig = context.getTestConfig();
		if (testConfig.noTest) return;

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

	protected getCompiler(context: ITestContext) {
		return context.getCompiler(this._options.name, this._options.compilerType);
	}
}
