import fs from "node:fs";
import path from "node:path";
import merge from "webpack-merge";
import { ECompilerEvent } from "../compiler";
import { readConfigFile } from "../helper";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import copyDiff from "../helper/legacy/copyDiff";
import { WatchRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions,
	TCompilerStatsCompilation
} from "../type";
import { compiler, findMultiCompilerBundle, getCompiler, run } from "./common";

type TWatchContext = {
	currentTriggerFilename: string | null;
	lastHash: string | null;
	step: string;
	tempDir: string;
	nativeWatcher: boolean;
	watchState: Record<string, any>;
};

// This file is used to port step number to rspack.config.js/webpack.config.js
const currentWatchStepModulePath = path.resolve(
	__dirname,
	"../helper/util/currentWatchStep"
);

export function createWatchInitialProcessor(
	name: string,
	tempDir: string,
	step: string,
	watchState: Record<string, any>,
	{
		incremental = false,
		nativeWatcher = false,
		ignoreNotFriendlyForIncrementalWarnings = false
	} = {}
) {
	const watchContext: TWatchContext = {
		currentTriggerFilename: null,
		lastHash: null,
		step,
		tempDir,
		nativeWatcher,
		watchState
	};

	return {
		before: async (context: ITestContext) => {
			context.setValue(name, "watchContext", watchContext);
		},
		config: async <T extends ECompilerType.Rspack>(context: ITestContext) => {
			const multiCompilerOptions = [];
			const caseOptions: TCompilerOptions<T>[] = readConfigFile(
				["rspack.config.js", "webpack.config.js"].map(i => context.getSource(i))
			);

			for (const [index, options] of caseOptions.entries()) {
				const compilerOptions = merge(
					defaultOptions!({
						incremental,
						ignoreNotFriendlyForIncrementalWarnings
					}),
					options
				);
				overrideOptions(
					index,
					context,
					compilerOptions,
					tempDir,
					nativeWatcher
				);
				multiCompilerOptions.push(compilerOptions);
			}

			const compilerOptions =
				multiCompilerOptions.length === 1
					? multiCompilerOptions[0]
					: multiCompilerOptions;
			const compiler = getCompiler(context, name);
			compiler.setOptions(compilerOptions as any);
			context.setValue(name, "multiCompilerOptions", multiCompilerOptions);
		},
		compiler: async (context: ITestContext) => {
			const c = await compiler(context, name);
			c!.hooks.invalid.tap("WatchTestCasesTest", (filename, mtime) => {
				watchContext.currentTriggerFilename = filename;
			});
		},
		build: async (context: ITestContext) => {
			const compiler = getCompiler(context, name);
			const currentWatchStepModule = require(currentWatchStepModulePath);
			currentWatchStepModule.step[name] = watchContext.step;
			fs.mkdirSync(watchContext.tempDir, { recursive: true });
			copyDiff(
				path.join(context.getSource(), watchContext.step),
				watchContext.tempDir,
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
		},
		run: async (env: ITestEnv, context: ITestContext) => {
			await run(env, context, name, (context: ITestContext) =>
				findMultiCompilerBundle(context, name, (index, context, options) =>
					findBundle(index, context, options, step)
				)
			);
		},
		check: async <T extends ECompilerType.Rspack>(
			env: ITestEnv,
			context: ITestContext
		) => {
			const testConfig = context.getTestConfig();
			if (testConfig.noTests) return;

			const errors: Array<{ message: string; stack?: string }> = (
				context.getError(name) || []
			).map(e => ({
				message: e.message,
				stack: e.stack
			}));
			const warnings: Array<{ message: string; stack?: string }> = [];
			const compiler = getCompiler(context, name);
			const stats = compiler.getStats();
			const options = compiler.getOptions();
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
						!checkStats(watchContext.step, getJsonStats(), getStringStats())
					) {
						throw new Error("stats check failed");
					}
				} else {
					// @ts-expect-error only one param
					if (!checkStats(watchContext.step)) {
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
					fs.existsSync(context.getSource(`${watchContext.step}/errors.js`)) ||
					fs.existsSync(
						context.getSource(`${watchContext.step}/warnings.js`)
					) ||
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
				path.join(context.getSource(), watchContext.step),
				{ errors },
				"error",
				"errors",
				"Error",
				options
			);

			await checkArrayExpectation(
				path.join(context.getSource(), watchContext.step),
				{ warnings },
				"warning",
				"warnings",
				"Warning",
				options
			);

			// clear error if checked
			if (fs.existsSync(context.getSource("errors.js"))) {
				context.clearError(name);
			}

			// check hash
			if (testConfig.writeStatsOuptut) {
				fs.renameSync(
					path.join(context.getDist(), "stats.txt"),
					path.join(context.getDist(), `stats.${watchContext.step}.txt`)
				);
			}
			if (testConfig.writeStatsJson) {
				fs.renameSync(
					path.join(context.getDist(), "stats.json"),
					path.join(context.getDist(), `stats.${watchContext.step}.json`)
				);
			}
		}
	};
}

export function createWatchStepProcessor(
	name: string,
	tempDir: string,
	step: string,
	watchState: Record<string, any>,
	{
		incremental = false,
		nativeWatcher = false,
		ignoreNotFriendlyForIncrementalWarnings = false
	} = {}
) {
	const processor = createWatchInitialProcessor(
		name,
		tempDir,
		step,
		watchState,
		{ incremental, ignoreNotFriendlyForIncrementalWarnings }
	);
	processor.compiler = async (context: ITestContext) => {
		// do nothing
	};
	processor.build = async (context: ITestContext) => {
		const watchContext = context.getValue(name, "watchContext") as any;
		const compiler = getCompiler(context, name);
		const currentWatchStepModule = require(currentWatchStepModulePath);
		currentWatchStepModule.step[name] = watchContext.step;
		const task = new Promise((resolve, reject) => {
			compiler.getEmitter().once(ECompilerEvent.Build, (e, stats) => {
				if (e) return reject(e);
				resolve(stats);
			});
		});
		// wait compiler to ready watch the files and diretories

		// Native Watcher using [notify](https://github.com/notify-rs/notify) to watch files.
		// After tests, notify will cost many milliseconds to watch in windows OS when jest run concurrently.
		// So we need to wait a while to ensure the watcher is ready.
		// If we don't wait, copyDiff will happen before the watcher is ready,
		// which will cause the compiler not rebuild when the files change.
		// The timeout is set to 400ms for windows OS and 100ms for other OS.
		// TODO: This is a workaround, we can remove it when notify support windows better.
		const timeout = nativeWatcher && process.platform === "win32" ? 400 : 100;
		await new Promise(resolve => setTimeout(resolve, timeout));
		copyDiff(path.join(context.getSource(), step), tempDir, false);
		await task;
	};
	return processor;
}

const creator = new BasicCaseCreator({
	clean: true,
	runner: WatchRunnerFactory,
	description: (name, index) => {
		return index === 0
			? `${name} should compile`
			: `should compile step ${index}`;
	},
	describe: false,
	steps: ({ name, src, temp }) => {
		const watchState = {};
		const runs = fs
			.readdirSync(src)
			.sort()
			.filter(name => fs.statSync(path.join(src, name)).isDirectory())
			.map(name => ({ name }));

		return runs.map((run, index) =>
			index === 0
				? createWatchInitialProcessor(name, temp!, run.name, watchState)
				: createWatchStepProcessor(name, temp!, run.name, watchState)
		);
	},
	concurrent: true
});

export function createWatchCase(
	name: string,
	src: string,
	dist: string,
	temp: string
) {
	creator.create(name, src, dist, temp);
}

function overrideOptions(
	index: number,
	context: ITestContext,
	options: TCompilerOptions<ECompilerType.Rspack>,
	tempDir: string,
	nativeWatcher: boolean
) {
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
	options.optimization ??= {};
	options.experiments ??= {};
	options.experiments.css ??= true;

	if (nativeWatcher) {
		(
			options as TCompilerOptions<ECompilerType.Rspack>
		).experiments!.nativeWatcher ??= true;
	}

	(
		options as TCompilerOptions<ECompilerType.Rspack>
	).experiments!.rspackFuture ??= {};
	(
		options as TCompilerOptions<ECompilerType.Rspack>
	).experiments!.rspackFuture!.bundlerInfo ??= {};
	(
		options as TCompilerOptions<ECompilerType.Rspack>
	).experiments!.rspackFuture!.bundlerInfo!.force ??= false;
	// test incremental: "safe" here, we test default incremental in Incremental-*.test.js
	(
		options as TCompilerOptions<ECompilerType.Rspack>
	).experiments!.incremental ??= "safe";

	if (!global.printLogger) {
		options.infrastructureLogging = {
			level: "error"
		};
	}
}

function findBundle(
	index: number,
	context: ITestContext,
	options: TCompilerOptions<ECompilerType.Rspack>,
	stepName: string
) {
	const testConfig = context.getTestConfig();

	if (typeof testConfig.findBundle === "function") {
		return testConfig.findBundle!(index, options, stepName);
	}
	return "./bundle.js";
}

function defaultOptions({
	incremental = false,
	ignoreNotFriendlyForIncrementalWarnings = false
} = {}): TCompilerOptions<ECompilerType.Rspack> {
	if (incremental) {
		return {
			experiments: {
				incremental: "advance"
			},
			ignoreWarnings: ignoreNotFriendlyForIncrementalWarnings
				? [/is not friendly for incremental/]
				: undefined
		};
	}
	return {};
}
