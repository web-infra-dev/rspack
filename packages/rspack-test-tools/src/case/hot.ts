import path from "node:path";
import rspack, { type StatsCompilation } from "@rspack/core";
import { isJavaScript } from "../helper";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import { TestHotUpdatePlugin } from "../helper/plugins";
import { LazyCompilationTestPlugin } from "../plugin";
import { WebRunner } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import {
	type ECompilerType,
	EDocumentType,
	type IModuleScope,
	type ITestContext,
	type ITestEnv,
	type ITestProcessor,
	type ITestRunner,
	type TCompilerOptions,
	type TCompilerStats,
	type TCompilerStatsCompilation,
	type THotUpdateContext
} from "../type";
import { build, check, compiler, config, getCompiler, run } from "./common";
import { cachedStats, type THotStepRuntimeData } from "./runner";

type TTarget = TCompilerOptions<ECompilerType.Rspack>["target"];

const creators: Map<
	TTarget,
	BasicCaseCreator<ECompilerType.Rspack>
> = new Map();

export function createHotProcessor(
	name: string,
	target: TTarget,
	incremental: boolean = false
): THotProcessor {
	const hotUpdateContext: THotUpdateContext = {
		updateIndex: 0,
		totalUpdates: 1,
		changedFiles: []
	};

	const processor = {
		config: async (context: ITestContext) => {
			const compiler = getCompiler(context, name);
			let options = defaultOptions(context, target, hotUpdateContext);
			options = await config(
				context,
				name,
				["rspack.config.js", "webpack.config.js"],
				options
			);
			overrideOptions(context, options, target, hotUpdateContext);
			if (incremental) {
				options.experiments ??= {};
				options.experiments.incremental ??= "advance-silent";
			}
			compiler.setOptions(options);
		},
		compiler: async (context: ITestContext) => {
			await compiler(context, name);
		},
		build: async (context: ITestContext) => {
			await build(context, name);
		},
		run: async (env: ITestEnv, context: ITestContext) => {
			context.setValue(name, "hotUpdateContext", hotUpdateContext);
			await run(env, context, name, context =>
				findBundle(context, name, target, hotUpdateContext)
			);
		},
		check: async (env: ITestEnv, context: ITestContext) => {
			await check(env, context, name);
		},
		afterAll: async (context: ITestContext) => {
			if (context.getTestConfig().checkSteps === false) {
				return;
			}

			if (hotUpdateContext.updateIndex + 1 !== hotUpdateContext.totalUpdates) {
				throw new Error(
					`Should run all hot steps (${hotUpdateContext.updateIndex + 1} / ${hotUpdateContext.totalUpdates}): ${name}`
				);
			}
		}
	} as THotProcessor;
	processor.hotUpdateContext = hotUpdateContext;
	return processor;
}

function getCreator(target: TTarget) {
	if (!creators.has(target)) {
		creators.set(
			target,
			new BasicCaseCreator({
				clean: true,
				describe: true,
				target,
				steps: ({ name, target }) => [
					createHotProcessor(name, target as TTarget)
				],
				runner: {
					key: (context: ITestContext, name: string, file: string) => name,
					runner: createHotRunner
				},
				concurrent: true
			})
		);
	}
	return creators.get(target)!;
}

export function createHotCase(
	name: string,
	src: string,
	dist: string,
	target: TCompilerOptions<ECompilerType.Rspack>["target"]
) {
	const creator = getCreator(target);
	creator.create(name, src, dist);
}

function defaultOptions(
	context: ITestContext,
	target: TTarget,
	updateOptions: THotUpdateContext
) {
	const options = {
		context: context.getSource(),
		mode: "development",
		devtool: false,
		output: {
			path: context.getDist(),
			filename: "bundle.js",
			chunkFilename: "[name].chunk.[fullhash].js",
			publicPath: "https://test.cases/path/",
			library: { type: "commonjs2" }
		},
		optimization: {
			moduleIds: "named"
		},
		target,
		experiments: {
			css: true,
			// test incremental: "safe" here, we test default incremental in Incremental-*.test.js
			incremental: "safe",
			rspackFuture: {
				bundlerInfo: {
					force: false
				}
			},
			inlineConst: true
		}
	} as TCompilerOptions<ECompilerType.Rspack>;

	options.plugins ??= [];
	(options as TCompilerOptions<ECompilerType.Rspack>).plugins!.push(
		new rspack.HotModuleReplacementPlugin(),
		new TestHotUpdatePlugin(updateOptions)
	);
	return options;
}

function overrideOptions(
	context: ITestContext,
	options: TCompilerOptions<ECompilerType.Rspack>,
	target: TTarget,
	updateOptions: THotUpdateContext
) {
	if (!options.entry) {
		options.entry = "./index.js";
	}

	options.module ??= {};
	for (const cssModuleType of ["css/auto", "css/module", "css"] as const) {
		options.module!.generator ??= {};
		options.module!.generator[cssModuleType] ??= {};
		options.module!.generator[cssModuleType]!.exportsOnly ??=
			target === "async-node";
	}
	options.module.rules ??= [];
	options.module.rules.push({
		use: [
			{
				loader: path.resolve(__dirname, "../helper/loaders/hot-update.js"),
				options: updateOptions
			}
		],
		enforce: "pre"
	});
	options.plugins ??= [];
	(options as TCompilerOptions<ECompilerType.Rspack>).plugins!.push(
		new rspack.LoaderOptionsPlugin(updateOptions)
	);
	if (!global.printLogger) {
		options.infrastructureLogging = {
			level: "error"
		};
	}

	if ((options as TCompilerOptions<ECompilerType.Rspack>).lazyCompilation) {
		(options as TCompilerOptions<ECompilerType.Rspack>).plugins!.push(
			new LazyCompilationTestPlugin()
		);
	}
}

function findBundle(
	context: ITestContext,
	name: string,
	target: TTarget,
	updateOptions: THotUpdateContext
): string | string[] {
	const compiler = context.getCompiler(name);
	if (!compiler) throw new Error("Compiler should exists when find bundle");

	const testConfig = context.getTestConfig();
	if (typeof testConfig.findBundle === "function") {
		return testConfig.findBundle!(
			updateOptions.updateIndex,
			compiler.getOptions()
		);
	}

	const files: string[] = [];
	const prefiles: string[] = [];

	const stats = compiler.getStats();
	if (!stats) throw new Error("Stats should exists when find bundle");
	const info = stats.toJson({ all: false, entrypoints: true });
	if (target === "web" || target === "webworker") {
		for (const file of info.entrypoints!.main.assets!) {
			if (isJavaScript(file.name)) {
				files.push(file.name);
			} else {
				prefiles.push(file.name);
			}
		}
	} else {
		const assets = info.entrypoints!.main.assets!.filter(s =>
			isJavaScript(s.name)
		);
		files.push(assets[assets.length - 1].name);
	}
	return [...prefiles, ...files];
}

type THotProcessor = ITestProcessor & {
	hotUpdateContext: THotUpdateContext;
};

export function createHotRunner<T extends ECompilerType = ECompilerType.Rspack>(
	context: ITestContext,
	name: string,
	file: string,
	env: ITestEnv
): ITestRunner {
	const compiler = context.getCompiler(name);
	const compilerOptions = compiler.getOptions() as TCompilerOptions<T>;
	const testConfig = context.getTestConfig();
	const source = context.getSource();
	const dist = context.getDist();
	const hotUpdateContext = context.getValue<THotUpdateContext>(
		name,
		"hotUpdateContext"
	)!;

	const next = async (
		callback?: (
			error: Error | null,
			stats?: TCompilerStatsCompilation<T>
		) => void
	) => {
		const usePromise = typeof callback === "function";
		try {
			hotUpdateContext.updateIndex++;
			const stats = await compiler.build();
			if (!stats) {
				throw new Error("Should generate stats during build");
			}
			const jsonStats = stats.toJson({
				// errorDetails: true
			});
			const compilerOptions = compiler.getOptions();

			const checker = context.getValue(
				name,
				jsonStats.errors?.length
					? "hotUpdateStepErrorChecker"
					: "hotUpdateStepChecker"
			) as (
				context: { updateIndex: number },
				stats: TCompilerStats<T>,
				runtime: THotStepRuntimeData
			) => void;
			if (checker) {
				checker(
					hotUpdateContext,
					stats as TCompilerStats<T>,
					runner.getGlobal("__HMR_UPDATED_RUNTIME__") as THotStepRuntimeData
				);
			}
			await checkArrayExpectation(
				source,
				jsonStats,
				"error",
				`errors${hotUpdateContext.updateIndex}`,
				"Error",
				compilerOptions
			);
			await checkArrayExpectation(
				source,
				jsonStats,
				"warning",
				`warnings${hotUpdateContext.updateIndex}`,
				"Warning",
				compilerOptions
			);
			if (usePromise) {
				// old callback style hmr cases
				callback(null, jsonStats as StatsCompilation);
			} else {
				// new promise style hmr cases
				return jsonStats as StatsCompilation;
			}
		} catch (e) {
			if (usePromise) {
				callback(e as Error);
			} else {
				throw e;
			}
		}
	};

	const nextHMR = async (m: any, options?: any) => {
		const jsonStats = await next();
		const updatedModules = await m.hot.check(options || true);
		if (!updatedModules) {
			throw new Error("No update available");
		}
		return jsonStats as StatsCompilation;
	};

	const runner = new WebRunner({
		dom: context.getValue(name, "documentType") || EDocumentType.JSDOM,
		env,
		stats: cachedStats(context, name),
		name: name,
		runInNewContext: false,
		testConfig: {
			documentType: testConfig.documentType || EDocumentType.Fake,
			...testConfig,
			moduleScope(
				ms: IModuleScope,
				stats?: TCompilerStatsCompilation<T>,
				options?: TCompilerOptions<T>
			) {
				const moduleScope = ms;
				if (typeof testConfig.moduleScope === "function") {
					testConfig.moduleScope(moduleScope, stats, compilerOptions);
				}
				moduleScope.NEXT = next;
				moduleScope.NEXT_HMR = nextHMR;
				return moduleScope;
			}
		},
		cachable: true,
		source,
		dist,
		compilerOptions
	});

	return runner;
}
