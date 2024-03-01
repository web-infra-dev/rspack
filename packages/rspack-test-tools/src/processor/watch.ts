import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestRunner,
	TCompilerOptions,
	TCompilerStats,
	TTestConfig
} from "../type";
import { MultiTaskProcessor } from "./multi";
import path from "path";
import fs from "fs";
import { startWatch } from "../helper";
import copyDiff from "../helper/legacy/copyDiff";
import { WatchRunner } from "../runner";
import EventEmitter from "events";

const currentWatchStepModulePath = path.resolve(
	__dirname,
	"../../../rspack/tests/helpers/currentWatchStep"
);

type TRspackExperiments = TCompilerOptions<ECompilerType.Rspack>["experiments"];
type TRspackOptimization =
	TCompilerOptions<ECompilerType.Rspack>["optimization"];

export interface IRspackWatchProcessorOptions {
	name: string;
	stepName: string;
	tempDir: string;
	testConfig: TTestConfig<ECompilerType.Rspack>;
	experiments?: TRspackExperiments;
	optimization?: TRspackOptimization;
	emitter: EventEmitter;
}

export class RspackWatchProcessor extends MultiTaskProcessor<ECompilerType.Rspack> {
	protected currentTriggerFilename: string | null = null;
	protected lastHash: string | null = null;

	constructor(protected _watchOptions: IRspackWatchProcessorOptions) {
		super({
			postOptions: RspackWatchProcessor.postOptions(_watchOptions),
			getCompiler: () => require("@rspack/core").rspack,
			getBundle: () => "bundle.js",
			configFiles: ["rspack.config.js", "webpack.config.js"],
			name: _watchOptions.name,
			testConfig: {
				timeout: 10000,
				..._watchOptions.testConfig
			}
		});
	}

	async compiler(context: ITestContext): Promise<void> {
		await super.compiler(context);
		context.compiler<ECompilerType.Rspack>((options, compiler) => {
			if (!compiler) return;
			compiler.hooks.invalid.tap("WatchTestCasesTest", (filename, mtime) => {
				this.currentTriggerFilename = filename;
			});
		}, this._options.name);
	}

	async build(context: ITestContext) {
		const currentWatchStepModule = require(currentWatchStepModulePath);
		currentWatchStepModule.step = this._watchOptions.stepName;
		fs.mkdirSync(this._watchOptions.tempDir, { recursive: true });
		copyDiff(
			path.join(context.getSource(), this._watchOptions.stepName),
			this._watchOptions.tempDir,
			true
		);
		const task = new Promise((resolve, reject) => {
			this._watchOptions.emitter.once("built", (e, stats) => {
				if (e) return reject(e);
				resolve(stats);
			});
		});
		startWatch<ECompilerType.Rspack>(
			context,
			1000,
			this._watchOptions.emitter,
			this._options.name
		);
		await task;
	}

	async check(env: ITestEnv, context: ITestContext) {
		await super.check(env, context);
		// check hash
		fs.renameSync(
			path.join(context.getDist(), "stats.txt"),
			path.join(context.getDist(), `stats.${this._watchOptions.stepName}.txt`)
		);
		fs.renameSync(
			path.join(context.getDist(), "stats.json"),
			path.join(context.getDist(), `stats.${this._watchOptions.stepName}.json`)
		);
	}

	async run(env: ITestEnv, context: ITestContext) {
		await super.run(env, context);
	}

	static postOptions({
		tempDir,
		name,
		experiments,
		optimization
	}: IRspackWatchProcessorOptions) {
		return (
			index: number,
			context: ITestContext,
			options: TCompilerOptions<ECompilerType.Rspack>
		): void => {
			if (!options.mode) options.mode = "development";
			if (!options.context) options.context = tempDir;
			if (!options.entry) options.entry = "./index.js";
			if (!options.target) options.target = "async-node";
			if (!options.output) options.output = {};
			if (!options.output.path) options.output.path = context.getDist();
			// CHANGE: The pathinfo is currently not supported in rspack
			// if (typeof options.output.pathinfo === "undefined")
			// 	options.output.pathinfo = true;
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
		};
	}

	protected createRunner(
		env: ITestEnv,
		context: ITestContext,
		options: TCompilerOptions<ECompilerType.Rspack>
	): ITestRunner | null {
		let runner: ITestRunner | null = null;
		context.stats<ECompilerType.Rspack>((_, stats) => {
			runner = new WatchRunner({
				env,
				stats: stats!,
				name: this._options.name,
				stepName: this._watchOptions.stepName,
				runInNewContext:
					options.target === "web" || options.target === "webworker",
				testConfig: this._options.testConfig,
				source: context.getSource(),
				dist: context.getDist(),
				compilerOptions: options
			});
		}, this._options.name);
		return runner;
	}
}

export interface IRspackWatchStepProcessorOptions {
	name: string;
	stepName: string;
	tempDir: string;
	testConfig: TTestConfig<ECompilerType.Rspack>;
	emitter: EventEmitter;
}

export class RspackWatchStepProcessor extends RspackWatchProcessor {
	constructor(protected _watchOptions: IRspackWatchStepProcessorOptions) {
		super(_watchOptions);
	}

	async compiler(context: ITestContext): Promise<void> {
		// do nothing
	}

	async build(context: ITestContext) {
		const currentWatchStepModule = require(currentWatchStepModulePath);
		currentWatchStepModule.step = this._watchOptions.stepName;
		const task = new Promise((resolve, reject) => {
			this._watchOptions.emitter.once("built", (e, stats) => {
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
