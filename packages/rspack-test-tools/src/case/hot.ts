import path from "node:path";
import rspack, { type StatsCompilation } from "@rspack/core";
import { isJavaScript } from "../helper";
import { HotUpdatePlugin } from "../helper/hot-update/plugin";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import { LazyCompilationTestPlugin } from "../plugin";
import { NodeRunner, WebRunner } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import type {
	ECompilerType,
	IModuleScope,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	ITestRunner,
	TCompilerOptions,
	TCompilerStats,
	TCompilerStatsCompilation
} from "../type";
import {
	afterExecute,
	build,
	check,
	compiler,
	config,
	getCompiler,
	run
} from "./common";
import { cachedStats, type THotStepRuntimeData } from "./runner";

type TTarget = TCompilerOptions<ECompilerType.Rspack>["target"];

const creators: Map<
	TTarget,
	BasicCaseCreator<ECompilerType.Rspack>
> = new Map();

export function createHotProcessor(
	name: string,
	src: string,
	temp: string,
	target: TTarget,
	incremental: boolean = false
): THotProcessor {
	const updatePlugin = new HotUpdatePlugin(src, temp);

	const processor = {
		before: async (context: ITestContext) => {
			await updatePlugin.initialize();
			context.setValue(name, "hotUpdatePlugin", updatePlugin);
		},
		config: async (context: ITestContext) => {
			const compiler = getCompiler(context, name);
			let options = defaultOptions(context, target);
			options = await config(
				context,
				name,
				["rspack.config.js", "webpack.config.js"],
				options
			);
			overrideOptions(context, options, target, updatePlugin);
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
			await run(env, context, name, context =>
				findBundle(context, name, target, updatePlugin)
			);
		},
		check: async (env: ITestEnv, context: ITestContext) => {
			await check(env, context, name);
		},
		after: async (context: ITestContext) => {
			await afterExecute(context, name);
		},
		afterAll: async (context: ITestContext) => {
			if (context.getTestConfig().checkSteps === false) {
				return;
			}

			const updateIndex = updatePlugin.getUpdateIndex();
			const totalUpdates = updatePlugin.getTotalUpdates();
			if (updateIndex + 1 !== totalUpdates) {
				throw new Error(
					`Should run all hot steps (${updateIndex + 1} / ${totalUpdates}): ${name}`
				);
			}
		}
	} as THotProcessor;
	processor.updatePlugin = updatePlugin;
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
				steps: ({ name, target, src, dist, temp }) => [
					createHotProcessor(
						name,
						src,
						temp || path.resolve(dist, "temp"),
						target as TTarget
					)
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
	temp: string,
	target: TCompilerOptions<ECompilerType.Rspack>["target"]
) {
	const creator = getCreator(target);
	creator.create(name, src, dist, temp);
}

function defaultOptions(context: ITestContext, target: TTarget) {
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
		new rspack.HotModuleReplacementPlugin()
	);
	return options;
}

function overrideOptions(
	context: ITestContext,
	options: TCompilerOptions<ECompilerType.Rspack>,
	target: TTarget,
	updatePlugin: HotUpdatePlugin
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
	options.plugins ??= [];
	(options as TCompilerOptions<ECompilerType.Rspack>).plugins!.push(
		updatePlugin
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
	updatePlugin: HotUpdatePlugin
): string | string[] {
	const compiler = context.getCompiler(name);
	if (!compiler) throw new Error("Compiler should exists when find bundle");

	const testConfig = context.getTestConfig();
	if (typeof testConfig.findBundle === "function") {
		return testConfig.findBundle!(
			updatePlugin.getUpdateIndex(),
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
	updatePlugin: HotUpdatePlugin;
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
	const updatePlugin = context.getValue<HotUpdatePlugin>(
		name,
		"hotUpdatePlugin"
	)!;

	const nextHMR = async (m: any, options?: any) => {
		await updatePlugin.goNext();
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
			updateIndex: number,
			stats: TCompilerStats<T>,
			runtime: THotStepRuntimeData
		) => void;
		if (checker) {
			checker(
				updatePlugin.getUpdateIndex(),
				stats as TCompilerStats<T>,
				runner.getGlobal("__HMR_UPDATED_RUNTIME__") as THotStepRuntimeData
			);
		}
		await checkArrayExpectation(
			source,
			jsonStats,
			"error",
			`errors${updatePlugin.getUpdateIndex()}`,
			"Error",
			compilerOptions
		);
		await checkArrayExpectation(
			source,
			jsonStats,
			"warning",
			`warnings${updatePlugin.getUpdateIndex()}`,
			"Warning",
			compilerOptions
		);
		const updatedModules = await m.hot.check(options || true);
		if (!updatedModules) {
			throw new Error("No update available");
		}
		return jsonStats as StatsCompilation;
	};

	const commonOptions = {
		env,
		stats: cachedStats(context, name),
		name: name,
		runInNewContext: false,
		testConfig: {
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
				moduleScope.NEXT_HMR = nextHMR;
				return moduleScope;
			}
		},
		cachable: true,
		source,
		dist,
		compilerOptions
	};
	let runner: ITestRunner;
	if (
		compilerOptions.target === "web" ||
		compilerOptions.target === "webworker"
	) {
		runner = new WebRunner({
			location: testConfig.location || "https://test.cases/path/index.html",
			...commonOptions
		});
	} else {
		runner = new NodeRunner(commonOptions);
	}
	return runner;
}
