import fs from "node:fs";
import path from "node:path";

import { readConfigFile } from "../helper";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	TCompiler,
	TCompilerMultiStats,
	TCompilerOptions,
	TCompilerStats
} from "../type";

export interface IBasicProcessorOptions<T extends ECompilerType> {
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
	compiler?: (context: ITestContext, compiler: TCompiler<T>) => Promise<void>;
	build?: (context: ITestContext, compiler: TCompiler<T>) => Promise<void>;
	check?: (
		env: ITestEnv,
		context: ITestContext,
		compiler: TCompiler<T>,
		stats: TCompilerStats<T> | TCompilerMultiStats<T> | null
	) => Promise<void>;
}

export class BasicProcessor<T extends ECompilerType> implements ITestProcessor {
	constructor(protected _options: IBasicProcessorOptions<T>) {}

	async config(context: ITestContext) {
		const compiler = this.getCompiler(context);
		if (typeof this._options.defaultOptions === "function") {
			compiler.setOptions(this._options.defaultOptions.call(this, context));
		}

		if (Array.isArray(this._options.configFiles)) {
			const fileOptions = readConfigFile<T>(
				this._options.configFiles.map(i => context.getSource(i))
			)[0];
			compiler.mergeOptions(fileOptions);
		}

		if (typeof this._options.overrideOptions === "function") {
			const compilerOptions = compiler.getOptions();
			this._options.overrideOptions.call(this, context, compilerOptions);
		}
	}

	async compiler(context: ITestContext) {
		const compiler = this.getCompiler(context);
		compiler.createCompiler();
		if (typeof this._options.compiler === "function") {
			await this._options.compiler.call(this, context, compiler.getCompiler()!);
		}
	}

	async build(context: ITestContext) {
		const compiler = this.getCompiler(context);
		if (typeof this._options.build === "function") {
			await this._options.build.call(this, context, compiler.getCompiler()!);
		} else {
			await compiler.build();
		}
	}

	async run(env: ITestEnv, context: ITestContext) {
		if (!this._options.runable) return;

		const testConfig = context.getTestConfig();
		if (testConfig.noTests) return;

		if (testConfig.documentType) {
			context.setValue(
				this._options.name,
				"documentType",
				testConfig.documentType
			);
		}

		const compiler = this.getCompiler(context);
		if (typeof testConfig.beforeExecute === "function") {
			testConfig.beforeExecute.call(this, compiler.getOptions());
		}

		let bundles: string[] | void | string;
		if (testConfig.bundlePath) {
			bundles = testConfig.bundlePath;
		} else if (typeof this._options.findBundle === "function") {
			bundles = this._options.findBundle.call(
				this,
				context,
				compiler.getOptions()
			);
		} else {
			bundles = [];
		}

		if (typeof bundles === "string") {
			bundles = [bundles];
		}
		if (!bundles || !bundles.length) {
			return;
		}

		for (const bundle of bundles!) {
			if (!bundle) {
				continue;
			}
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
			testConfig.afterExecute.call(this, compiler.getOptions());
		}
	}

	async check(env: ITestEnv, context: ITestContext) {
		const testConfig = context.getTestConfig();
		if (testConfig.noTests) return;

		const compiler = this.getCompiler(context);
		if (typeof this._options.check === "function") {
			const stats = compiler.getStats();
			await this._options.check.call(
				this,
				env,
				context,
				compiler.getCompiler()!,
				stats
			);
			return;
		}

		const errors: Array<{ message: string; stack?: string }> = (
			context.getError(this._options.name) || []
		).map(e => ({
			message: e.message,
			stack: e.stack
		}));
		const warnings: Array<{ message: string; stack?: string }> = [];

		const stats = compiler.getStats();
		const options = compiler.getOptions();
		if (stats) {
			if (testConfig.writeStatsOuptut) {
				fs.writeFileSync(
					path.join(context.getDist(), "stats.txt"),
					stats.toString({
						preset: "verbose",
						colors: false
					}),
					"utf-8"
				);
			}

			if (testConfig.writeStatsJson) {
				const jsonStats = stats.toJson({
					errorDetails: true
				});
				fs.writeFileSync(
					path.join(context.getDist(), "stats.json"),
					JSON.stringify(jsonStats, null, 2),
					"utf-8"
				);
			}

			if (
				fs.existsSync(context.getSource("errors.js")) ||
				fs.existsSync(context.getSource("warnings.js")) ||
				stats.hasErrors() ||
				stats.hasWarnings()
			) {
				const statsJson = stats.toJson({
					errorDetails: true
				});
				if (statsJson.errors) {
					errors.push(...statsJson.errors);
				}
				if (statsJson.warnings) {
					warnings.push(...statsJson.warnings);
				}
			}
		}
		await checkArrayExpectation(
			context.getSource(),
			{ errors },
			"error",
			"errors",
			"Error",
			options
		);

		await checkArrayExpectation(
			context.getSource(),
			{ warnings },
			"warning",
			"warnings",
			"Warning",
			options
		);

		// clear error if checked
		if (fs.existsSync(context.getSource("errors.js"))) {
			context.clearError(this._options.name);
		}
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
