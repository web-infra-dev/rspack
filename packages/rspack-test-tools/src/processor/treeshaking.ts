import { ECompilerType, ITestContext, TCompilerOptions } from "../type";
import path from "path";
import fs from "fs-extra";
import { merge } from "webpack-merge";
import { rspack } from "@rspack/core";
import { SnapshotProcessor } from "./snapshot";

export interface IRspackTreeShakingProcessorOptions {
	name: string;
	snapshot: string;
	type: "new" | "builtin";
}

export class RspackTreeShakingProcessor extends SnapshotProcessor<ECompilerType.Rspack> {
	constructor(
		protected _treeShakingOptions: IRspackTreeShakingProcessorOptions
	) {
		super({
			snapshot: _treeShakingOptions.snapshot,
			compilerType: ECompilerType.Rspack,
			defaultOptions: RspackTreeShakingProcessor.defaultOptions,
			overrideOptions: RspackTreeShakingProcessor.overrideOptions(
				_treeShakingOptions.type
			),
			name: _treeShakingOptions.name,
			runable: false
		});
	}

	static defaultOptions(
		context: ITestContext
	): TCompilerOptions<ECompilerType.Rspack> {
		let defaultOptions: TCompilerOptions<ECompilerType.Rspack> = {
			entry: {
				main: {
					import: "./index",
					runtime: "runtime"
				}
			},
			output: {
				filename: "[name].js",
				chunkFilename: "[name].js",
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
			defaultOptions = merge(defaultOptions, require(testConfigFile));
		}

		const defineOptions = (defaultOptions.builtins as any)?.define;
		if (defineOptions) {
			defaultOptions.plugins!.push(new rspack.DefinePlugin(defineOptions));
			delete (defaultOptions.builtins as any)?.define;
		}

		return defaultOptions;
	}

	static overrideOptions(type: IRspackTreeShakingProcessorOptions["type"]) {
		return (
			context: ITestContext,
			options: TCompilerOptions<ECompilerType.Rspack>
		) => {
			options.target = options.target || ["web", "es2022"];
			if (type === "new") {
				options.optimization ??= {};
				options.optimization.providedExports = true;
				options.optimization.innerGraph = true;
				options.optimization.usedExports = true;

				options.experiments ??= {};
				options.experiments.rspackFuture ??= {};
				options.experiments.rspackFuture.newTreeshaking = true;

				options.builtins ??= {};
				options.builtins.treeShaking = false;
			}
		};
	}
}
