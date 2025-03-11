import fs from "node:fs";
import path from "node:path";
import { merge } from "webpack-merge";

import { ECompilerEvent } from "../compiler";
import { readConfigFile } from "../helper";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import copyDiff from "../helper/legacy/copyDiff";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions,
	TCompilerStatsCompilation
} from "../type";
import { type IMultiTaskProcessorOptions, MultiTaskProcessor } from "./multi";

// This file is used to port step number to rspack.config.js/webpack.config.js
const currentWatchStepModulePath = path.resolve(
	__dirname,
	"../helper/util/currentWatchStep"
);

type TRspackExperiments = TCompilerOptions<ECompilerType>["experiments"];
type TRspackOptimization = TCompilerOptions<ECompilerType>["optimization"];

export interface IWatchProcessorOptions<T extends ECompilerType>
	extends IMultiTaskProcessorOptions<T> {
	stepName: string;
	tempDir: string;
	experiments?: TRspackExperiments;
	optimization?: TRspackOptimization;
}

export class WatchProcessor<
	T extends ECompilerType
> extends MultiTaskProcessor<T> {
	protected currentTriggerFilename: string | null = null;
	protected lastHash: string | null = null;

	constructor(
		protected _watchOptions: IWatchProcessorOptions<T>,
		protected _watchState: Record<string, any>
	) {
		super({
			overrideOptions: WatchProcessor.overrideOptions<T>(_watchOptions),
			findBundle: WatchProcessor.findBundle<T>,
			..._watchOptions
		});
	}

	async compiler(context: ITestContext): Promise<void> {
		await super.compiler(context);
		const compiler = this.getCompiler(context).getCompiler();
		compiler!.hooks.invalid.tap("WatchTestCasesTest", (filename, mtime) => {
			this.currentTriggerFilename = filename;
		});
	}

	async build(context: ITestContext) {
		const compiler = this.getCompiler(context);
		const currentWatchStepModule = require(currentWatchStepModulePath);
		currentWatchStepModule.step[this._options.name] =
			this._watchOptions.stepName;
		fs.mkdirSync(this._watchOptions.tempDir, { recursive: true });
		copyDiff(
			path.join(context.getSource(), this._watchOptions.stepName),
			this._watchOptions.tempDir,
			true
		);
		const task = new Promise((resolve, reject) => {
			compiler.getEmitter().once(ECompilerEvent.Build, (e, stats) => {
				if (e) return reject(e);
				resolve(stats);
			});
		});
		compiler.watch();
		await task;
	}

	async run(env: ITestEnv, context: ITestContext) {
		context.setValue(
			this._options.name,
			"watchStepName",
			this._watchOptions.stepName
		);
		context.setValue(this._options.name, "watchState", this._watchState);
		await super.run(env, context);
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
		const checkStats = testConfig.checkStats || (() => true);

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

			const getJsonStats = (() => {
				let cached: TCompilerStatsCompilation<T> | null = null;
				return () => {
					if (!cached) {
						cached = stats.toJson({
							errorDetails: true
						});
					}
					return cached;
				};
			})();
			const getStringStats = (() => {
				let cached: string | null = null;
				return () => {
					if (!cached) {
						cached = stats.toString({
							logging: "verbose"
						});
					}
					return cached;
				};
			})();
			if (checkStats.length > 1) {
				if (
					!checkStats(
						this._watchOptions.stepName,
						getJsonStats(),
						getStringStats()
					)
				) {
					throw new Error("stats check failed");
				}
			} else {
				// @ts-expect-error only one param
				if (!checkStats(this._watchOptions.stepName)) {
					throw new Error("stats check failed");
				}
			}
			if (testConfig.writeStatsJson) {
				fs.writeFileSync(
					path.join(context.getDist(), "stats.json"),
					JSON.stringify(getJsonStats(), null, 2),
					"utf-8"
				);
			}
			if (
				fs.existsSync(
					context.getSource(`${this._watchOptions.stepName}/errors.js`)
				) ||
				fs.existsSync(
					context.getSource(`${this._watchOptions.stepName}/warnings.js`)
				)
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
			path.join(context.getSource(), this._watchOptions.stepName),
			{ errors },
			"error",
			"errors",
			"Error"
		);

		await checkArrayExpectation(
			path.join(context.getSource(), this._watchOptions.stepName),
			{ warnings },
			"warning",
			"warnings",
			"Warning"
		);

		// clear error if checked
		if (fs.existsSync(context.getSource("errors.js"))) {
			context.clearError(this._options.name);
		}

		// check hash
		if (testConfig.writeStatsOuptut) {
			fs.renameSync(
				path.join(context.getDist(), "stats.txt"),
				path.join(context.getDist(), `stats.${this._watchOptions.stepName}.txt`)
			);
		}
		if (testConfig.writeStatsJson) {
			fs.renameSync(
				path.join(context.getDist(), "stats.json"),
				path.join(
					context.getDist(),
					`stats.${this._watchOptions.stepName}.json`
				)
			);
		}
	}

	async config(context: ITestContext) {
		this.multiCompilerOptions = [];
		const caseOptions: TCompilerOptions<T>[] = Array.isArray(
			this._multiOptions.configFiles
		)
			? readConfigFile(
					this._multiOptions.configFiles!.map(i => context.getSource(i))
				)
			: [{}];

		for (const [index, options] of caseOptions.entries()) {
			const compilerOptions = merge(
				typeof this._multiOptions.defaultOptions === "function"
					? this._multiOptions.defaultOptions!(index, context)
					: {},
				options
			);

			if (typeof this._multiOptions.overrideOptions === "function") {
				this._multiOptions.overrideOptions!(index, context, compilerOptions);
			}

			this.multiCompilerOptions.push(compilerOptions);
		}

		const compilerOptions =
			this.multiCompilerOptions.length === 1
				? this.multiCompilerOptions[0]
				: this.multiCompilerOptions;
		const compiler = this.getCompiler(context);
		compiler.setOptions(compilerOptions as any);
	}

	static overrideOptions<T extends ECompilerType>({
		tempDir,
		name,
		experiments,
		optimization
	}: IWatchProcessorOptions<T>) {
		return (
			index: number,
			context: ITestContext,
			options: TCompilerOptions<ECompilerType>
		): void => {
			if (!options.mode) options.mode = "development";
			if (!options.context) options.context = tempDir;
			if (!options.entry) options.entry = "./index.js";
			if (!options.target) options.target = "async-node";
			if (!options.devtool) options.devtool = false;
			if (!options.output) options.output = {};
			if (!options.output.path) options.output.path = context.getDist();
			if (typeof options.output.pathinfo === "undefined")
				options.output.pathinfo = false;
			if (!options.output.filename) options.output.filename = "bundle.js";
			if (options.cache && (options.cache as any).type === "filesystem") {
				const cacheDirectory = path.join(tempDir, ".cache");
				(options.cache as any).cacheDirectory = cacheDirectory;
				(options.cache as any).name = `config-${index}`;
			}
			if (experiments) {
				if (!options.experiments) options.experiments = {};
				for (const key of Object.keys(experiments) as Array<
					keyof TRspackExperiments
				>) {
					if (options.experiments[key] === undefined)
						options.experiments[key] = experiments[key];
				}
			}
			if (optimization) {
				if (!options.optimization) options.optimization = {};
				for (const key of Object.keys(optimization) as Array<
					keyof TRspackOptimization
				>) {
					if (options.optimization[key] === undefined)
						options.optimization[key] = optimization[key];
				}
			}

			(options as TCompilerOptions<ECompilerType.Rspack>).experiments ??= {};
			(options as TCompilerOptions<ECompilerType.Rspack>).experiments!.css ??=
				true;
			(
				options as TCompilerOptions<ECompilerType.Rspack>
			).experiments!.rspackFuture ??= {};
			(
				options as TCompilerOptions<ECompilerType.Rspack>
			).experiments!.rspackFuture!.bundlerInfo ??= {};
			(
				options as TCompilerOptions<ECompilerType.Rspack>
			).experiments!.rspackFuture!.bundlerInfo!.force ??= false;

			if (!global.printLogger) {
				options.infrastructureLogging = {
					level: "error"
				};
			}
		};
	}

	static findBundle<T extends ECompilerType>(
		this: IWatchProcessorOptions<T>,
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	) {
		const testConfig = context.getTestConfig();

		if (typeof testConfig.findBundle === "function") {
			return testConfig.findBundle!(index, options, this.stepName);
		}
		return "./bundle.js";
	}
}

export interface IWatchStepProcessorOptions<T extends ECompilerType>
	extends Omit<IWatchProcessorOptions<T>, "experiments" | "optimization"> {}

export class WatchStepProcessor<
	T extends ECompilerType
> extends WatchProcessor<T> {
	constructor(
		protected _watchOptions: IWatchStepProcessorOptions<T>,
		protected _watchState: Record<string, any>
	) {
		super(_watchOptions, _watchState);
	}

	async compiler(context: ITestContext): Promise<void> {
		// do nothing
	}

	async build(context: ITestContext) {
		const compiler = this.getCompiler(context);
		const currentWatchStepModule = require(currentWatchStepModulePath);
		currentWatchStepModule.step[this._options.name] =
			this._watchOptions.stepName;
		const task = new Promise((resolve, reject) => {
			compiler.getEmitter().once(ECompilerEvent.Build, (e, stats) => {
				if (e) return reject(e);
				resolve(stats);
			});
		});
		copyDiff(
			path.join(context.getSource(), this._watchOptions.stepName),
			this._watchOptions.tempDir,
			false
		);
		await task;
	}
}
