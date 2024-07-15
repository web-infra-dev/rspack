import { rspack } from "@rspack/core";
import fs from "fs-extra";
import { merge } from "webpack-merge";

import {
	ECompilerType,
	type ITestContext,
	type TCompilerOptions
} from "../type";
import { type ISnapshotProcessorOptions, SnapshotProcessor } from "./snapshot";

export interface IBuiltinProcessorOptions<T extends ECompilerType>
	extends Omit<ISnapshotProcessorOptions<T>, "runable"> {}

export class BuiltinProcessor<
	T extends ECompilerType
> extends SnapshotProcessor<T> {
	constructor(protected _builtinOptions: IBuiltinProcessorOptions<T>) {
		super({
			defaultOptions: BuiltinProcessor.defaultOptions,
			runable: false,
			..._builtinOptions
		});
	}

	static defaultOptions<T extends ECompilerType>(
		this: BuiltinProcessor<T>,
		context: ITestContext
	): TCompilerOptions<T> {
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
				css: true,
				futureDefaults: true,
				rspackFuture: {
					bundlerInfo: {
						force: false
					}
				}
			},
			devtool: false,
			context: context.getSource(),
			plugins: []
		} as TCompilerOptions<T>;

		if (this._options.compilerType === ECompilerType.Rspack) {
			let rspackDefaultOptions =
				defaultOptions as TCompilerOptions<ECompilerType.Rspack>;
			const testConfigFile = context.getSource("rspack.config.js");
			if (fs.existsSync(testConfigFile)) {
				const caseOptions = require(testConfigFile);
				if (caseOptions.entry) {
					delete rspackDefaultOptions.entry;
				}
				rspackDefaultOptions = merge(rspackDefaultOptions, caseOptions);
			}

			// TODO: remove builtin compatible code
			const defineOptions = (rspackDefaultOptions as any).builtins?.define;
			if (defineOptions) {
				rspackDefaultOptions.plugins!.push(
					new rspack.DefinePlugin(defineOptions)
				);
			}

			const provideOptions = (rspackDefaultOptions as any).builtins?.provide;
			if (provideOptions) {
				rspackDefaultOptions.plugins!.push(
					new rspack.ProvidePlugin(provideOptions)
				);
			}

			const htmlOptions = (rspackDefaultOptions as any).builtins?.html;
			if (htmlOptions) {
				if (Array.isArray(htmlOptions)) {
					for (const item of htmlOptions) {
						rspackDefaultOptions.plugins!.push(
							new rspack.HtmlRspackPlugin(item)
						);
					}
				} else {
					rspackDefaultOptions.plugins!.push(
						new rspack.HtmlRspackPlugin(htmlOptions)
					);
				}
			}

			delete (rspackDefaultOptions as any).builtins;

			defaultOptions = rspackDefaultOptions as TCompilerOptions<T>;
		}

		return defaultOptions;
	}
}
