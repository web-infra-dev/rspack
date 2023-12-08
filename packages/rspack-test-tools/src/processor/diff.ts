import {
	ECompilerType,
	ITestContext,
	ITestProcessor,
	TCompareModules,
	TCompilerOptions,
	TFileCompareResult,
	TModuleCompareResult
} from "../type";
import path from "path";
import { createModulePlaceholderPlugin } from "../webpack/module-placeholder-plugin";
import {
	IFormatCodeOptions,
	compareFile,
	replaceRuntimeModuleName
} from "../compare";
import { readConfigFile, runBuild } from "../helper";
import deepmerge from "deepmerge";

export interface IDiffProcessorOptions extends IFormatCodeOptions {
	webpackPath: string;
	rspackPath: string;
	files?: string[];
	modules?: TCompareModules;
	runtimeModules?: TCompareModules;
	onCompareFile?: (file: string, result: TFileCompareResult) => void;
	onCompareModules?: (file: string, results: TModuleCompareResult[]) => void;
	onCompareRuntimeModules?: (
		file: string,
		results: TModuleCompareResult[]
	) => void;
}

export class DiffProcessor implements ITestProcessor {
	private hashes: string[] = [];
	constructor(private options: IDiffProcessorOptions) {}

	async config(context: ITestContext) {
		this.setCompilerOptions(
			ECompilerType.Rspack,
			["rspack.config.js", "webpack.config.js"],
			context
		);
		this.setCompilerOptions(
			ECompilerType.Webpack,
			["webpack.config.js", "rspack.config.js"],
			context
		);
	}
	async compiler(context: ITestContext) {
		const rspack = require(this.options.rspackPath).rspack;
		context.compiler<ECompilerType.Rspack>(
			options => rspack({ ...options }),
			ECompilerType.Rspack
		);
		const webpack = require(this.options.webpackPath).webpack;
		context.compiler<ECompilerType.Webpack>(
			options => webpack({ ...options }),
			ECompilerType.Webpack
		);
	}
	async build(context: ITestContext) {
		const rspackStats = await runBuild<ECompilerType.Rspack>(
			context,
			ECompilerType.Rspack
		);
		const webpackStats = await runBuild<ECompilerType.Webpack>(
			context,
			ECompilerType.Webpack
		);
		//TODO: handle chunk hash and content hash
		rspackStats?.hash && this.hashes.push(rspackStats?.hash);
		webpackStats?.hash && this.hashes.push(webpackStats?.hash);
	}
	async check(context: ITestContext) {
		const dist = context.getDist();
		for (let file of this.options.files!) {
			const rspackDist = path.join(dist, ECompilerType.Rspack, file);
			const webpackDist = path.join(dist, ECompilerType.Webpack, file);
			const result = compareFile(rspackDist, webpackDist, {
				modules: this.options.modules,
				runtimeModules: this.options.runtimeModules,
				format: this.createFormatOptions(),
				renameModule: replaceRuntimeModuleName
			});
			if (typeof this.options.onCompareFile === "function") {
				this.options.onCompareFile(file, result);
			}
			if (
				typeof this.options.onCompareModules === "function" &&
				result.modules["modules"]
			) {
				this.options.onCompareModules(file, result.modules["modules"]);
			}
			if (
				typeof this.options.onCompareRuntimeModules === "function" &&
				result.modules["runtimeModules"]
			) {
				this.options.onCompareRuntimeModules(
					file,
					result.modules["runtimeModules"]
				);
			}
		}
	}

	private setCompilerOptions<T extends ECompilerType>(
		type: T,
		configFiles: string[],
		context: ITestContext
	) {
		const source = context.getSource();
		const dist = context.getDist();
		context.options<T>(
			options =>
				this.setDefaultOptions<T>(options, type, source, path.join(dist, type)),
			type
		);
		context.options<T>(
			options => readConfigFile<T>(source, configFiles, options),
			type
		);
	}

	private setDefaultOptions<T extends ECompilerType>(
		options: TCompilerOptions<T>,
		type: T,
		src: string,
		dist: string
	) {
		let result = deepmerge<TCompilerOptions<T>>(options, {
			entry: path.join(src, "./src/index.js"),
			context: src,
			output: {
				filename: "bundle.js",
				chunkFilename: "[name].chunk.js"
			},
			mode: "development",
			devtool: false,
			optimization: {
				chunkIds: "named",
				moduleIds: "named"
			}
		});
		if (type === ECompilerType.Webpack) {
			result = deepmerge<TCompilerOptions<ECompilerType.Webpack>>(
				result as TCompilerOptions<ECompilerType.Webpack>,
				{
					output: {
						pathinfo: false,
						environment: {
							arrowFunction: false,
							bigIntLiteral: false,
							const: false,
							destructuring: false,
							dynamicImport: false,
							dynamicImportInWorker: false,
							forOf: false,
							globalThis: false,
							module: false,
							optionalChaining: false,
							templateLiteral: false
						},
						path: dist
					},
					optimization: {
						mangleExports: false,
						concatenateModules: false
					},
					plugins: [createModulePlaceholderPlugin(this.options.webpackPath)]
				},
				{
					arrayMerge: (a, b) => [...a, ...b]
				}
			) as TCompilerOptions<T>;
		}
		if (type === ECompilerType.Rspack) {
			result = deepmerge<TCompilerOptions<ECompilerType.Rspack>>(
				result as TCompilerOptions<ECompilerType.Rspack>,
				{
					output: {
						path: dist
					},
					optimization: {
						mangleExports: false
					},
					experiments: {
						rspackFuture: {
							disableTransformByDefault: true
						}
					}
				}
			) as TCompilerOptions<T>;
		}
		return result;
	}

	private createFormatOptions() {
		const formatOptions: IFormatCodeOptions = {
			ignoreModuleArguments: this.options.ignoreModuleArguments,
			ignoreModuleId: this.options.ignoreModuleId,
			ignorePropertyQuotationMark: this.options.ignorePropertyQuotationMark,
			ignoreBlockOnlyStatement: this.options.ignoreBlockOnlyStatement,
			ignoreSwcHelpersPath: this.options.ignoreSwcHelpersPath,
			ignoreObjectPropertySequence: this.options.ignoreObjectPropertySequence,
			replacements: this.options.replacements || {}
		};
		for (let hash of this.hashes) {
			formatOptions.replacements![hash] = "fullhash";
		}
		return formatOptions;
	}
}
