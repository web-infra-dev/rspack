import { rspack } from "@rspack/core";
import fs from "fs-extra";
import { merge } from "webpack-merge";

import { ECompilerType, ITestContext, TCompilerOptions } from "../type";
import { ISnapshotProcessorOptions, SnapshotProcessor } from "./snapshot";

export interface IBuiltinProcessorOptions<T extends ECompilerType>
	extends Omit<ISnapshotProcessorOptions<T>, "defaultOptions" | "runable"> {}

export class BuiltinProcessor<
	T extends ECompilerType
> extends SnapshotProcessor<T> {
	constructor(protected _builtinOptions: IBuiltinProcessorOptions<T>) {
		super({
			defaultOptions: BuiltinProcessor.defaultOptions<T>(
				_builtinOptions.compilerType
			),
			runable: false,
			..._builtinOptions
		});
	}

	static defaultOptions<T extends ECompilerType>(
		compilerType: T
	): (context: ITestContext) => TCompilerOptions<T> {
		return (context: ITestContext) => {
			let defaultOptions = {
				entry: {
					main: {
						import: "./index"
					}
				},
				output: {
					publicPath: "/",
					path: context.getDist(),
					filename: "[name].js",
					chunkFilename: "[name].js",
					chunkFormat: "array-push",
					cssFilename: "[name].css",
					cssChunkFilename: "[name].css",
					assetModuleFilename: "[hash][ext][query]",
					sourceMapFilename: "[file].map",
					chunkLoadingGlobal: "webpackChunkwebpack",
					chunkLoading: "jsonp",
					uniqueName: "__rspack_test__",
					enabledLibraryTypes: ["system"],
					strictModuleErrorHandling: false,
					iife: true,
					module: false,
					asyncChunks: true,
					scriptType: false,
					globalObject: "self",
					importFunctionName: "import",
					wasmLoading: "fetch",
					webassemblyModuleFilename: "[hash].module.wasm",
					workerChunkLoading: "import-scripts",
					workerWasmLoading: "fetch"
				},
				module: {
					rules: [
						{
							test: /\.json$/,
							type: "json"
						},
						{
							test: /\.mjs$/,
							type: "js/esm"
						},
						{
							test: /\.cjs$/,
							type: "js/dynamic"
						},
						{
							test: /\.js$/,
							type: "js/auto"
						},
						{
							test: /\.css$/,
							type: "css"
						},
						{
							test: /\.wasm$/,
							type: "webassembly/async"
						}
					]
				},
				node: {
					__dirname: "mock",
					__filename: "mock",
					global: "warn"
				},
				optimization: {
					runtimeChunk: {
						name: "runtime"
					},
					minimize: false,
					removeAvailableModules: true,
					removeEmptyChunks: true,
					moduleIds: "named",
					chunkIds: "named",
					sideEffects: false,
					mangleExports: false,
					usedExports: false,
					concatenateModules: false,
					nodeEnv: false
				},
				resolve: {
					extensions: [
						".js",
						".jsx",
						".ts",
						".tsx",
						".json",
						".d.ts",
						".css",
						".wasm"
					]
				},
				resolveLoader: {
					extensions: [".js"]
				},
				experiments: {
					futureDefaults: true
				},
				devtool: false,
				context: context.getSource(),
				plugins: [],
				builtins: {
					treeShaking: false
				}
			} as TCompilerOptions<T>;

			if (compilerType === ECompilerType.Rspack) {
				let rspackDefaultOptions =
					defaultOptions as TCompilerOptions<ECompilerType.Rspack>;
				const testConfigFile = context.getSource("rspack.config.js");
				if (fs.existsSync(testConfigFile)) {
					let caseOptions = require(testConfigFile);
					if (caseOptions.entry) {
						delete rspackDefaultOptions.entry;
					}
					rspackDefaultOptions = merge(rspackDefaultOptions, caseOptions);
				}

				// TODO: remove builtin compatible code
				const defineOptions = (rspackDefaultOptions.builtins as any)?.define;
				if (defineOptions) {
					rspackDefaultOptions.plugins!.push(
						new rspack.DefinePlugin(defineOptions)
					);
					delete (rspackDefaultOptions.builtins as any)?.define;
				}

				const provideOptions = (rspackDefaultOptions.builtins as any)?.provide;
				if (provideOptions) {
					rspackDefaultOptions.plugins!.push(
						new rspack.ProvidePlugin(provideOptions)
					);
					delete (rspackDefaultOptions.builtins as any)?.provide;
				}

				const htmlOptions = (rspackDefaultOptions.builtins as any)?.html;
				if (htmlOptions) {
					if (Array.isArray(htmlOptions)) {
						for (let item of htmlOptions) {
							rspackDefaultOptions.plugins!.push(
								new rspack.HtmlRspackPlugin(item)
							);
						}
					} else {
						rspackDefaultOptions.plugins!.push(
							new rspack.HtmlRspackPlugin(htmlOptions)
						);
					}
					delete (rspackDefaultOptions.builtins as any)?.html;
				}

				defaultOptions = rspackDefaultOptions as TCompilerOptions<T>;
			}

			return defaultOptions;
		};
	}
}
