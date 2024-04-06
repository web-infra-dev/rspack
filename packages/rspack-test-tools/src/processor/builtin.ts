import { ECompilerType, ITestContext, TCompilerOptions } from "../type";
import fs from "fs-extra";
import { merge } from "webpack-merge";
import { EntryDescription, rspack } from "@rspack/core";
import { SnapshotProcessor } from "./snapshot";

export interface IRspackBuiltinProcessorOptions {
	name: string;
	snapshot: string;
}

export class RspackBuiltinProcessor extends SnapshotProcessor<ECompilerType.Rspack> {
	constructor(protected _builtinOptions: IRspackBuiltinProcessorOptions) {
		super({
			snapshot: _builtinOptions.snapshot,
			compilerType: ECompilerType.Rspack,
			defaultOptions: RspackBuiltinProcessor.defaultOptions,
			name: _builtinOptions.name,
			runable: false
		});
	}

	static defaultOptions(
		context: ITestContext
	): TCompilerOptions<ECompilerType.Rspack> {
		let defaultOptions: TCompilerOptions<ECompilerType.Rspack> = {
			entry: {
				main: {
					import: "./index"
				}
			},
			output: {
				path: context.getDist(),
				filename: "[name].js",
				chunkFilename: "[name].js",
				chunkFormat: "array-push",
				cssFilename: "[name].css",
				cssChunkFilename: "[name].css",
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
				importFunctionName: "import"
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
			devtool: false,
			context: context.getSource(),
			plugins: []
		};

		const testConfigFile = context.getSource("test.config.js");
		if (fs.existsSync(testConfigFile)) {
			let caseOptions = require(testConfigFile);
			if (caseOptions.entry) {
				delete defaultOptions.entry;
			}
			defaultOptions = merge(defaultOptions, caseOptions);
		}

		const defineOptions = (defaultOptions.builtins as any)?.define;
		if (defineOptions) {
			defaultOptions.plugins!.push(new rspack.DefinePlugin(defineOptions));
			delete (defaultOptions.builtins as any)?.define;
		}

		return defaultOptions;
	}
}
