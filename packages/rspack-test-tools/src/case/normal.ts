import fs from "node:fs";
import path from "node:path";
import {
	type Compiler,
	HotModuleReplacementPlugin,
	type RspackOptions
} from "@rspack/core";
import {
	BasicCaseCreator,
	type IBasicCaseCreatorOptions
} from "../test/creator";
import type { ITestContext, ITestEnv } from "../type";
import { afterExecute, build, check, compiler, config, run } from "./common";
import { createRunner } from "./runner";

const NORMAL_CASES_ROOT = path.resolve(__TEST_PATH__, "normalCases");

const createCaseOptions = (
	hot: boolean,
	mode?: "development" | "production"
): IBasicCaseCreatorOptions => {
	return {
		clean: true,
		describe: false,
		steps: ({ name }) => [
			{
				config: async (context: ITestContext) => {
					const compiler = context.getCompiler();
					let options = defaultOptions(
						context,
						{
							plugins: hot ? [new HotModuleReplacementPlugin()] : []
						},
						mode
					);
					options = await config(
						context,
						name,
						["rspack.config.js", "webpack.config.js"],
						options
					);
					overrideOptions(context, options);
					compiler.setOptions(options);
				},
				compiler: async (context: ITestContext) => {
					await compiler(context, name);
				},
				build: async (context: ITestContext) => {
					await build(context, name);
				},
				run: async (env: ITestEnv, context: ITestContext) => {
					await run(env, context, name, findBundle);
				},
				check: async (env: ITestEnv, context: ITestContext) => {
					await check(env, context, name);
				},
				after: async (context: ITestContext) => {
					await afterExecute(context, name);
				}
			}
		],
		runner: {
			key: (context: ITestContext, name: string, file: string) => name,
			runner: createRunner
		},
		concurrent: true
	};
};

const creator = new BasicCaseCreator(createCaseOptions(false));
export function createNormalCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}

const hotCreator = new BasicCaseCreator(createCaseOptions(true));
export function createHotNormalCase(name: string, src: string, dist: string) {
	hotCreator.create(name, src, dist);
}

const devCreator = new BasicCaseCreator(
	createCaseOptions(false, "development")
);
export function createDevNormalCase(name: string, src: string, dist: string) {
	devCreator.create(name, src, dist);
}

const prodCreator = new BasicCaseCreator(
	createCaseOptions(false, "production")
);
export function createProdNormalCase(name: string, src: string, dist: string) {
	prodCreator.create(name, src, dist);
}

function findBundle(context: ITestContext, options: RspackOptions) {
	const testConfig = context.getTestConfig();

	if (typeof testConfig.findBundle === "function") {
		return testConfig.findBundle!(0, options);
	}

	const filename = options.output?.filename;
	return typeof filename === "string" ? filename : undefined;
}

function defaultOptions(
	context: ITestContext,
	compilerOptions: RspackOptions,
	mode?: "development" | "production"
) {
	let testConfig: RspackOptions = {};
	const testConfigPath = path.join(context.getSource(), "test.config.js");
	if (fs.existsSync(testConfigPath)) {
		testConfig = require(testConfigPath);
	}
	const TerserPlugin = require("terser-webpack-plugin");
	const terserForTesting = new TerserPlugin({
		parallel: false
	});
	return {
		amd: {},
		context: NORMAL_CASES_ROOT,
		entry: `./${path.relative(NORMAL_CASES_ROOT, context.getSource())}/`,
		target: compilerOptions?.target || "async-node",
		devtool: mode === "development" ? false : compilerOptions?.devtool,
		mode: compilerOptions?.mode || mode || "none",
		optimization: compilerOptions?.mode
			? {
					emitOnErrors: true,
					minimizer: [terserForTesting],
					minimize: false,
					...testConfig.optimization
				}
			: {
					removeAvailableModules: true,
					removeEmptyChunks: true,
					mergeDuplicateChunks: true,
					// CHANGE: rspack does not support `flagIncludedChunks` yet.
					// flagIncludedChunks: true,
					sideEffects: true,
					providedExports: true,
					usedExports: true,
					mangleExports: true,
					// CHANGE: rspack does not support `emitOnErrors` yet.
					emitOnErrors: true,
					concatenateModules: !!testConfig?.optimization?.concatenateModules,
					innerGraph: true,
					// CHANGE: size is not supported yet
					// moduleIds: "size",
					// chunkIds: "size",
					moduleIds: "named",
					chunkIds: "named",
					minimize: false,
					minimizer: [terserForTesting],
					...compilerOptions?.optimization
				},
		// CHANGE: rspack does not support `performance` yet.
		// performance: {
		// 	hints: false
		// },
		node: {
			__dirname: "mock",
			__filename: "mock"
		},
		cache: compilerOptions?.cache && {
			// cacheDirectory,
			...(compilerOptions.cache as any)
		},
		output: {
			pathinfo: "verbose",
			path: context.getDist(),
			filename: compilerOptions?.module ? "bundle.mjs" : "bundle.js"
		},
		resolve: {
			modules: ["web_modules", "node_modules"],
			mainFields: ["webpack", "browser", "web", "browserify", "main"],
			aliasFields: ["browser"],
			extensions: [".webpack.js", ".web.js", ".js", ".json"]
		},
		resolveLoader: {
			modules: ["web_loaders", "web_modules", "node_loaders", "node_modules"],
			mainFields: ["webpackLoader", "webLoader", "loader", "main"],
			extensions: [".webpack-loader.js", ".web-loader.js", ".loader.js", ".js"]
		},
		module: {
			rules: [
				{
					test: /\.coffee$/,
					loader: "coffee-loader"
				},
				{
					test: /\.pug/,
					loader: "@webdiscus/pug-loader"
				},
				{
					test: /\.wat$/i,
					loader: "wast-loader",
					type: "webassembly/async"
				}
			]
		},
		plugins: (compilerOptions?.plugins || [])
			.concat(testConfig.plugins || [])
			.concat(function (this: Compiler) {
				this.hooks.compilation.tap("TestCasesTest", compilation => {
					const hooks: never[] = [
						// CHANGE: the following hooks are not supported yet, so comment it out
						// "optimize",
						// "optimizeModules",
						// "optimizeChunks",
						// "afterOptimizeTree",
						// "afterOptimizeAssets"
					];

					for (const hook of hooks) {
						(compilation.hooks[hook] as any).tap("TestCasesTest", () =>
							(compilation as any).checkConstraints()
						);
					}
				});
			}),
		experiments: {
			css: false,
			rspackFuture: {
				bundlerInfo: {
					force: false
				}
			},
			asyncWebAssembly: true,
			topLevelAwait: true,
			inlineConst: true,
			// CHANGE: rspack does not support `backCompat` yet.
			// backCompat: false,
			// CHANGE: Rspack enables `css` by default.
			// Turning off here to fallback to webpack's default css processing logic.
			...(compilerOptions?.module ? { outputModule: true } : {})
		}
	} as RspackOptions;
}

function overrideOptions(context: ITestContext, options: RspackOptions) {
	if (!global.printLogger) {
		options.infrastructureLogging = {
			level: "error"
		};
	}
}
