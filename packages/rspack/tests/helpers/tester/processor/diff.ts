import {
	ECompilerType,
	ITestContext,
	ITestProcessor,
	TCompilerOptions
} from "../type";
import fs from "fs-extra";
import path from "path";
import { webpack } from "webpack";
import { rspack } from "@rspack/core";
import { createModulePlaceholderPlugin } from "../plugin/module-placeholder";
import {
	parseModules,
	readConfigFile,
	runBuild,
	replaceRuntimeModuleName,
	IFormatCodeOptions,
	TCompareModules,
	TModuleCompareResult,
	compareModules
} from "../helper";

export const OUTPUT_MAIN_FILE = "bundle.js";

export interface IDiffProcessorOptions extends IFormatCodeOptions {
	files?: string[];
	modules?: TCompareModules;
	onCompareModules?: (file: string, results: TModuleCompareResult[]) => void;
	runtimeModules?: TCompareModules;
	onCompareRuntimeModules?: (
		file: string,
		results: TModuleCompareResult[]
	) => void;
}

export class DiffProcessor implements ITestProcessor {
	private hashes: string[] = [];
	constructor(private options: IDiffProcessorOptions) {}

	async config(context: ITestContext) {
		const source = context.getSource();

		// base configuration
		context.options<ECompilerType.Rspack>(
			options =>
				this.setDefaultOptions<ECompilerType.Rspack>(
					options,
					ECompilerType.Rspack,
					source,
					this.getRspackDist(context)
				),
			"rspack"
		);
		context.options<ECompilerType.Webpack>(
			options =>
				this.setDefaultOptions<ECompilerType.Webpack>(
					options,
					ECompilerType.Webpack,
					source,
					this.getWebpackDist(context)
				),
			"webpack"
		);
		context.options<ECompilerType.Rspack>(
			options =>
				readConfigFile<ECompilerType.Rspack>(
					source,
					["rspack.config.js", "webpack.config.js"],
					options
				),
			"rspack"
		);
		context.options<ECompilerType.Webpack>(
			options =>
				readConfigFile<ECompilerType.Webpack>(
					source,
					["webpack.config.js"],
					options
				),
			"webpack"
		);
	}
	async compiler(context: ITestContext) {
		context.compiler<ECompilerType.Rspack>(
			options => rspack({ ...options }),
			"rspack"
		);
		context.compiler<ECompilerType.Webpack>(
			options => webpack({ ...options }),
			"webpack"
		);
	}
	async build(context: ITestContext) {
		const rspackStats = await runBuild<ECompilerType.Rspack>(context, "rspack");
		const webpackStats = await runBuild<ECompilerType.Webpack>(
			context,
			"webpack"
		);
		rspackStats?.hash && this.hashes.push(rspackStats?.hash);
		webpackStats?.hash && this.hashes.push(webpackStats?.hash);
	}
	async check(context: ITestContext) {
		for (let file of this.options.files!) {
			const rspackModules = parseModules(
				replaceRuntimeModuleName(
					fs.readFileSync(path.join(this.getRspackDist(context), file), "utf-8")
				)
			);
			const webpackModules = parseModules(
				fs.readFileSync(path.join(this.getWebpackDist(context), file), "utf-8")
			);
			const formatOptions = this.createFormatOptions();
			if (
				this.options.modules &&
				typeof this.options.onCompareModules === "function"
			) {
				this.options.onCompareModules(
					file,
					compareModules(
						this.options.modules,
						rspackModules.modules,
						webpackModules.modules,
						formatOptions
					)
				);
			}
			if (
				this.options.runtimeModules &&
				typeof this.options.onCompareRuntimeModules === "function"
			) {
				this.options.onCompareRuntimeModules(
					file,
					compareModules(
						this.options.runtimeModules,
						rspackModules.runtimeModules,
						webpackModules.runtimeModules,
						formatOptions
					)
				);
			}
		}
	}

	private setDefaultOptions<T extends ECompilerType>(
		options: TCompilerOptions<T>,
		type: T,
		src: string,
		dist: string
	) {
		// output options
		options.output ??= {};
		options.output.filename ??= OUTPUT_MAIN_FILE;
		options.output.chunkFilename ??= "[name].chunk.js";
		// entry options
		options.entry = path.join(src, "./src/index.js");
		options.context = src;
		// production
		options.mode = "development";
		options.devtool = false;
		// optimization
		options.optimization = {
			chunkIds: "named",
			moduleIds: "named"
		};
		if (type === ECompilerType.Webpack) {
			const webpackOptions = options as TCompilerOptions<ECompilerType.Webpack>;
			webpackOptions.plugins ??= [];
			webpackOptions.plugins!.push(createModulePlaceholderPlugin());
			webpackOptions.output!.pathinfo = false;
			webpackOptions.output!.environment = {
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
			};
			webpackOptions.output!.path = dist;
			webpackOptions.optimization!.concatenateModules = false;
			webpackOptions.optimization!.mangleExports = false;
		}
		if (type === ECompilerType.Rspack) {
			const rspackOptions = options as TCompilerOptions<ECompilerType.Rspack>;
			rspackOptions.experiments = {
				rspackFuture: {
					disableTransformByDefault: true
				}
			};
			rspackOptions.output!.path = dist;
		}
		return options;
	}

	private createFormatOptions() {
		const formatOptions: IFormatCodeOptions = {
			ignoreModuleArugments: this.options.ignoreModuleArugments,
			ignoreModuleId: this.options.ignoreModuleId,
			ignorePropertyQuotationMark: this.options.ignorePropertyQuotationMark,
			replacements: this.options.replacements || {}
		};
		for (let hash of this.hashes) {
			formatOptions.replacements![hash] = "fullhash";
		}
		return formatOptions;
	}

	private getWebpackDist(context: ITestContext) {
		const dist = context.getDist();
		return path.join(dist, "webpack");
	}

	private getRspackDist(context: ITestContext) {
		const dist = context.getDist();
		return path.join(dist, "rspack");
	}
}
