import path from "node:path";
import { rspack } from "@rspack/core";
import fs from "fs-extra";
import { merge } from "webpack-merge";
import { isJavaScript } from "../helper";
import { BasicCaseCreator } from "../test/creator";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions
} from "../type";
import { build, checkSnapshot, compiler, getCompiler } from "./common";

export function defaultOptions<T extends ECompilerType.Rspack>(
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
					type: "javascript/esm"
				},
				{
					test: /\.cjs$/,
					type: "javascript/dynamic"
				},
				{
					test: /\.js$/,
					type: "javascript/auto"
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

	const testConfigFile = context.getSource("rspack.config.js");
	if (fs.existsSync(testConfigFile)) {
		const caseOptions = require(testConfigFile);
		if (caseOptions.entry) {
			delete defaultOptions.entry;
		}
		defaultOptions = merge(defaultOptions, caseOptions);
	}

	// TODO: remove builtin compatible code
	const defineOptions = (defaultOptions as any).builtins?.define;
	if (defineOptions) {
		defaultOptions.plugins!.push(new rspack.DefinePlugin(defineOptions));
	}

	const provideOptions = (defaultOptions as any).builtins?.provide;
	if (provideOptions) {
		defaultOptions.plugins!.push(new rspack.ProvidePlugin(provideOptions));
	}

	const htmlOptions = (defaultOptions as any).builtins?.html;
	if (htmlOptions) {
		if (Array.isArray(htmlOptions)) {
			for (const item of htmlOptions) {
				defaultOptions.plugins!.push(new rspack.HtmlRspackPlugin(item));
			}
		} else {
			defaultOptions.plugins!.push(new rspack.HtmlRspackPlugin(htmlOptions));
		}
	}

	delete (defaultOptions as any).builtins;

	if (!global.printLogger) {
		defaultOptions.infrastructureLogging = {
			level: "error"
		};
	}

	return defaultOptions;
}

const FILTERS: Record<string, (file: string) => boolean> = {
	"plugin-css": (file: string) => file.endsWith(".css"),
	"plugin-css-modules": (file: string) =>
		file.endsWith(".css") || (isJavaScript(file) && !file.includes("runtime")),
	"plugin-html": (file: string) => file.endsWith(".html")
};

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	description(name) {
		return `${name} should match snapshot`;
	},
	steps: ({ name, src }) => {
		const cat = path.basename(path.dirname(src));
		const filter = FILTERS[cat];
		return [
			{
				config: async (context: ITestContext) => {
					const compiler = getCompiler(context, name);
					compiler.setOptions(defaultOptions(context));
				},
				compiler: async (context: ITestContext) => {
					await compiler(context, name);
				},
				build: async (context: ITestContext) => {
					await build(context, name);
				},
				run: async (env: ITestEnv, context: ITestContext) => {
					// no need to run, just check snapshot
				},
				check: async (env: ITestEnv, context: ITestContext) => {
					await checkSnapshot(env, context, name, "output.snap.txt", filter);
				}
			}
		];
	},
	concurrent: true
});

export function createBuiltinCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
