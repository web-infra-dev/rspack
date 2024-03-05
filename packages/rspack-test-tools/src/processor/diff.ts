import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	TCompareModules,
	TCompilerOptions,
	TFileCompareResult,
	TModuleCompareResult
} from "../type";
import path from "path";
import {
	IFormatCodeOptions,
	compareFile,
	replaceRuntimeModuleName
} from "../compare";
import { RspackDiffConfigPlugin, WebpackDiffConfigPlugin } from "../plugin";
import { BasicTaskProcessor } from "./basic";
import { readConfigFile } from "..";

export interface IDiffProcessorOptions extends IFormatCodeOptions {
	webpackPath: string;
	rspackPath: string;
	files?: string[];
	modules?: TCompareModules;
	runtimeModules?: TCompareModules;
	bootstrap?: boolean;
	detail?: boolean;
	onCompareFile?: (file: string, result: TFileCompareResult) => void;
	onCompareModules?: (file: string, results: TModuleCompareResult[]) => void;
	onCompareRuntimeModules?: (
		file: string,
		results: TModuleCompareResult[]
	) => void;
}
export class DiffProcessor implements ITestProcessor {
	private hashes: string[] = [];
	private webpack: BasicTaskProcessor<ECompilerType.Webpack>;
	private rspack: BasicTaskProcessor<ECompilerType.Rspack>;
	constructor(private options: IDiffProcessorOptions) {
		this.webpack = new BasicTaskProcessor<ECompilerType.Webpack>({
			preOptions: context =>
				this.getDefaultOptions(
					ECompilerType.Webpack,
					context.getSource(),
					path.join(context.getDist(), ECompilerType.Webpack)
				),
			getCompiler: () => require(this.options.webpackPath).webpack,
			getBundle: () => {},
			name: ECompilerType.Webpack,
			getCompilerOptions: context =>
				readConfigFile<ECompilerType.Webpack>(context.getSource(), [
					"webpack.config.js",
					"rspack.config.js"
				])[0],
			testConfig: {}
		});

		this.rspack = new BasicTaskProcessor<ECompilerType.Rspack>({
			preOptions: context =>
				this.getDefaultOptions(
					ECompilerType.Rspack,
					context.getSource(),
					path.join(context.getDist(), ECompilerType.Rspack)
				),
			getCompiler: () => require(this.options.rspackPath).rspack,
			getBundle: () => {},
			name: ECompilerType.Rspack,
			getCompilerOptions: context =>
				readConfigFile<ECompilerType.Rspack>(context.getSource(), [
					"rspack.config.js",
					"webpack.config.js"
				])[0],
			testConfig: {}
		});
	}

	async config(context: ITestContext) {
		await this.webpack.config(context);
		await this.rspack.config(context);
	}
	async compiler(context: ITestContext) {
		await this.webpack.compiler(context);
		await this.rspack.compiler(context);
	}
	async build(context: ITestContext) {
		await this.webpack.build(context);
		await this.rspack.build(context);
	}
	async check(env: ITestEnv, context: ITestContext) {
		context.stats((compiler, stats) => {
			//TODO: handle chunk hash and content hash
			stats?.hash && this.hashes.push(stats?.hash);
		}, ECompilerType.Webpack);
		context.stats((compiler, stats) => {
			//TODO: handle chunk hash and content hash
			stats?.hash && this.hashes.push(stats?.hash);
		}, ECompilerType.Rspack);

		const dist = context.getDist();
		for (let file of this.options.files!) {
			const rspackDist = path.join(dist, ECompilerType.Rspack, file);
			const webpackDist = path.join(dist, ECompilerType.Webpack, file);
			const result = compareFile(rspackDist, webpackDist, {
				modules: this.options.modules,
				runtimeModules: this.options.runtimeModules,
				format: this.createFormatOptions(),
				renameModule: replaceRuntimeModuleName,
				bootstrap: this.options.bootstrap,
				detail: this.options.detail
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

	private getDefaultOptions<T extends ECompilerType>(
		type: T,
		src: string,
		dist: string
	) {
		return {
			entry: path.join(src, "./src/index.js"),
			context: src,
			output: {
				path: dist,
				filename: "bundle.js",
				chunkFilename: "[name].chunk.js"
			},
			plugins: [
				type === ECompilerType.Webpack && new WebpackDiffConfigPlugin(),
				type === ECompilerType.Rspack && new RspackDiffConfigPlugin()
			].filter(Boolean)
		} as TCompilerOptions<T>;
	}

	private createFormatOptions() {
		const formatOptions: IFormatCodeOptions = {
			ignoreModuleArguments: this.options.ignoreModuleArguments,
			ignoreModuleId: this.options.ignoreModuleId,
			ignorePropertyQuotationMark: this.options.ignorePropertyQuotationMark,
			ignoreBlockOnlyStatement: this.options.ignoreBlockOnlyStatement,
			ignoreIfCertainCondition: this.options.ignoreIfCertainCondition,
			ignoreSwcHelpersPath: this.options.ignoreSwcHelpersPath,
			ignoreObjectPropertySequence: this.options.ignoreObjectPropertySequence,
			ignoreCssFilePath: this.options.ignoreCssFilePath,
			replacements: this.options.replacements || {}
		};
		for (let hash of this.hashes) {
			formatOptions.replacements![hash] = "fullhash";
		}
		return formatOptions;
	}
}
