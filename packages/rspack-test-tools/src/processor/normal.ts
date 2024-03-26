import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestRunner,
	TCompilerOptions,
	TTestConfig
} from "../type";
import { BasicTaskProcessor } from "./basic";
import path from "path";
import fs from "fs";

const CWD = process.cwd();

export interface IRspackNormalProcessorOptions {
	name: string;
	root: string;
	compilerOptions?: TCompilerOptions<ECompilerType.Rspack>;
	runable: boolean;
}

export class RspackNormalProcessor extends BasicTaskProcessor<ECompilerType.Rspack> {
	constructor(protected _normalOptions: IRspackNormalProcessorOptions) {
		super({
			compilerType: ECompilerType.Rspack,
			findBundle: (context, options) => {
				const filename = options.output?.filename;
				return typeof filename === "string" ? filename : undefined;
			},
			defaultOptions: RspackNormalProcessor.defaultOptions(_normalOptions),
			name: _normalOptions.name,
			runable: _normalOptions.runable
		});
	}

	async before(context: ITestContext) {
		//TODO: remove this
		process.chdir(path.resolve(__dirname, "../../../rspack"));
	}

	async after(context: ITestContext) {
		process.chdir(CWD);
	}

	static defaultOptions({
		compilerOptions,
		root
	}: IRspackNormalProcessorOptions) {
		return (context: ITestContext): TCompilerOptions<ECompilerType.Rspack> => {
			let testConfig: TCompilerOptions<ECompilerType.Rspack> = {};
			const testConfigPath = path.join(context.getSource(), "test.config.js");
			if (fs.existsSync(testConfigPath)) {
				testConfig = require(testConfigPath);
			}
			const TerserPlugin = require("terser-webpack-plugin");
			const terserForTesting = new TerserPlugin({
				parallel: false
			});
			return {
				context: root,
				entry: "./" + path.relative(root, context.getSource()) + "/",
				target: compilerOptions?.target || "async-node",
				devtool: compilerOptions?.devtool,
				mode: compilerOptions?.mode || "none",
				optimization: compilerOptions?.mode
					? {
							// emitOnErrors: true,
							minimizer: [terserForTesting],
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
							// emitOnErrors: true,
							concatenateModules:
								!!testConfig?.optimization?.concatenateModules,
							innerGraph: true,
							// CHANGE: size is not supported yet
							// moduleIds: "size",
							// chunkIds: "size",
							moduleIds: "named",
							chunkIds: "named",
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
					modules: [
						"web_loaders",
						"web_modules",
						"node_loaders",
						"node_modules"
					],
					mainFields: ["webpackLoader", "webLoader", "loader", "main"],
					extensions: [
						".webpack-loader.js",
						".web-loader.js",
						".loader.js",
						".js"
					]
				},
				module: {
					rules: [
						{
							test: /\.coffee$/,
							loader: "coffee-loader"
						},
						{
							test: /\.pug/,
							loader: "pug-loader"
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
					.concat(function () {
						this.hooks.compilation.tap("TestCasesTest", compilation => {
							[
								// CHANGE: the follwing hooks are not supported yet, so comment it out
								// "optimize",
								// "optimizeModules",
								// "optimizeChunks",
								// "afterOptimizeTree",
								// "afterOptimizeAssets"
							].forEach(hook => {
								(compilation.hooks[hook] as any).tap("TestCasesTest", () =>
									(compilation as any).checkConstraints()
								);
							});
						});
					}),
				experiments: {
					asyncWebAssembly: true,
					topLevelAwait: true,
					// CHANGE: rspack does not support `backCompat` yet.
					// backCompat: false,
					// CHANGE: Rspack enables `css` by default.
					// Turning off here to fallback to webpack's default css processing logic.

					rspackFuture: testConfig?.experiments?.rspackFuture ?? {
						newTreeshaking: true
					},
					css: false,
					...(compilerOptions?.module ? { outputModule: true } : {})
				}
				// infrastructureLogging: compilerOptions?.cache && {
				//   debug: true,
				//   console: createLogger(infraStructureLog)
				// }
			};
		};
	}
}
