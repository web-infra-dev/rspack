import path from "node:path";
import rspack from "@rspack/core";
import { isJavaScript } from "../helper";
import { HotUpdatePlugin } from "../helper/hot-update";
import { CacheRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	TCompilerOptions
} from "../type";
import { build, check, compiler, config, getCompiler, run } from "./common";

type TTarget = TCompilerOptions<ECompilerType.Rspack>["target"];

const creators: Map<
	TTarget,
	BasicCaseCreator<ECompilerType.Rspack>
> = new Map();

function defaultOptions(
	context: ITestContext,
	temp: string,
	target: TTarget
): TCompilerOptions<ECompilerType.Rspack> {
	const options = {
		context: temp,
		mode: "production",
		cache: true,
		devtool: false,
		output: {
			path: context.getDist(),
			filename: "bundle.js",
			chunkFilename: "[name].chunk.[fullhash].js",
			publicPath: "https://test.cases/path/",
			library: { type: "commonjs2" }
		},
		optimization: {
			moduleIds: "named",
			emitOnErrors: true
		},
		target,
		experiments: {
			css: true,
			rspackFuture: {
				bundlerInfo: {
					force: false
				}
			},
			inlineConst: true,
			lazyBarrel: true
		}
	} as TCompilerOptions<ECompilerType.Rspack>;

	options.plugins ??= [];
	(options as TCompilerOptions<ECompilerType.Rspack>).plugins!.push(
		new rspack.HotModuleReplacementPlugin()
	);

	return options;
}

function overrideOptions<T extends ECompilerType>(
	options: TCompilerOptions<T>,
	temp: string,
	target: TTarget,
	updatePlugin: HotUpdatePlugin
): void {
	if (!options.entry) {
		options.entry = "./index.js";
	}

	// rewrite context to temp dir
	options.context = temp;
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
}

function findBundle(
	name: string,
	target: TTarget,
	context: ITestContext
): string[] {
	const files: string[] = [];
	const prefiles: string[] = [];
	const compiler = getCompiler(context, name);
	if (!compiler) throw new Error("Compiler should exists when find bundle");
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

function createCacheProcessor(
	name: string,
	src: string,
	temp: string,
	target: TTarget
): ITestProcessor {
	const updatePlugin = new HotUpdatePlugin(src, temp);
	return {
		before: async (context: ITestContext) => {
			await updatePlugin.initialize();
			context.setValue(name, "hotUpdateContext", updatePlugin);
		},
		config: async (context: ITestContext) => {
			const compiler = getCompiler(context, name);
			let options = defaultOptions(context, temp, target);
			options = await config(
				context,
				name,
				["rspack.config.js", "webpack.config.js"].map(i =>
					path.resolve(temp, i)
				),
				options
			);
			overrideOptions(options, temp, target, updatePlugin);
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
				findBundle(name, target, context)
			);
		},
		check: async (env: ITestEnv, context: ITestContext) => {
			await check(env, context, name);
		},
		afterAll: async (context: ITestContext) => {
			const updateIndex = updatePlugin.getUpdateIndex();
			const totalUpdates = updatePlugin.getTotalUpdates();
			if (updateIndex + 1 !== totalUpdates) {
				throw new Error(
					`Should run all hot steps (${updateIndex + 1} / ${totalUpdates}): ${name}`
				);
			}
		}
	} as ITestProcessor;
}

function getCreator(target: TTarget) {
	if (!creators.has(target)) {
		creators.set(
			target,
			new BasicCaseCreator({
				clean: true,
				describe: true,
				target,
				steps: ({ name, src, target, temp }) => [
					createCacheProcessor(name, src, temp!, target as TTarget)
				],
				runner: CacheRunnerFactory,
				concurrent: true
			})
		);
	}
	return creators.get(target)!;
}

export function createCacheCase(
	name: string,
	src: string,
	dist: string,
	target: TCompilerOptions<ECompilerType.Rspack>["target"],
	temp: string
) {
	const creator = getCreator(target);
	creator.create(name, src, dist, temp);
}
