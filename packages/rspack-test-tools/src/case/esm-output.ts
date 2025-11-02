import rspack, { type RspackOptions } from "@rspack/core";
import { BasicCaseCreator } from "../test/creator";
import type { ITestContext, ITestEnv } from "../type";
import {
	afterExecute,
	build,
	check,
	checkSnapshot,
	compiler,
	configMultiCompiler,
	findMultiCompilerBundle,
	run
} from "./common";
import { createMultiCompilerRunner, getMultiCompilerRunnerKey } from "./runner";

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	testConfig: testConfig => {
		const oldModuleScope = testConfig.moduleScope;
		testConfig.moduleScope = (ms, stats) => {
			let res = ms;
			// TODO: modify runner module scope based on stats here
			if (typeof oldModuleScope === "function") {
				res = oldModuleScope(ms, stats);
			}
			return res;
		};
	},
	steps: ({ name }) => [
		{
			config: async (context: ITestContext) => {
				configMultiCompiler(
					context,
					name,
					["rspack.config.cjs", "rspack.config.js", "webpack.config.js"],
					defaultOptions,
					(_index, context, options) => {
						const testConfig = context.getTestConfig();
						if (testConfig.esmLibPluginOptions) {
							let target;

							const otherPlugins =
								options.plugins?.filter(plugin => {
									const isTarget =
										plugin instanceof rspack.experiments.EsmLibraryPlugin;
									if (isTarget) {
										target = plugin;
									}
									return !isTarget;
								}) ?? [];

							options.plugins = [
								...otherPlugins,
								new rspack.experiments.EsmLibraryPlugin({
									...target!.options,
									...testConfig.esmLibPluginOptions
								})
							];
						}
					}
				);
			},
			compiler: async (context: ITestContext) => {
				await compiler(context, name);
			},
			build: async (context: ITestContext) => {
				await build(context, name);
			},
			run: async (env: ITestEnv, context: ITestContext) => {
				await run(env, context, name, (context: ITestContext) =>
					findMultiCompilerBundle(context, name, (_index, context, options) => {
						const testConfig = context.getTestConfig();
						if (typeof testConfig.findBundle === "function") {
							return testConfig.findBundle(_index, options);
						}
						if (options.output?.filename === "[name].mjs") {
							return ["main.mjs"];
						} else {
							return [options.output!.filename as string];
						}
					})
				);
			},
			check: async (env: ITestEnv, context: ITestContext) => {
				await check(env, context, name);
				await checkSnapshot(env, context, name, "esm.snap.txt");
			},
			after: async (context: ITestContext) => {
				await afterExecute(context, name);
			}
		}
	],
	runner: {
		key: getMultiCompilerRunnerKey,
		runner: createMultiCompilerRunner
	},
	concurrent: 1
});

const defaultOptions = (
	_index: number,
	context: ITestContext
): RspackOptions => ({
	context: context.getSource(),
	mode: "production",
	target: "async-node",
	devtool: false,
	entry: "./index.js",
	cache: false,
	output: {
		path: context.getDist(),
		filename: "[name].mjs",
		pathinfo: true,
		module: true
	},
	bail: true,
	optimization: {
		minimize: false,
		moduleIds: "named",
		chunkIds: "named",
		runtimeChunk: "single"
	},
	plugins: [new rspack.experiments.EsmLibraryPlugin()],
	experiments: {
		css: true,
		rspackFuture: {
			bundlerInfo: {
				force: false
			}
		},
		outputModule: true
	}
});

export function createEsmOutputCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
