import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestRunner,
	TCompilerOptions
} from "../type";
import { BasicTaskProcessor, IBasicProcessorOptions } from "./basic";
import path from "path";
import { StatsCompilation, rspack } from "@rspack/core";
import { readConfigFile } from "../helper";
import { HotRunner } from "../runner";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";

export interface IRspackHotProcessorOptions {
	name: string;
	target: TCompilerOptions<ECompilerType.Rspack>["target"];
}

type TUpdateOptions = {
	updateIndex: number;
};

export class RspackHotProcessor extends BasicTaskProcessor<ECompilerType.Rspack> {
	protected updateOptions: TUpdateOptions;
	protected runner: ITestRunner | null = null;

	constructor(protected _hotOptions: IRspackHotProcessorOptions) {
		const fakeUpdateLoaderOptions: TUpdateOptions = {
			updateIndex: 0
		};
		super({
			defaultOptions: RspackHotProcessor.defaultOptions(
				_hotOptions,
				fakeUpdateLoaderOptions
			),
			overrideOptions: RspackHotProcessor.overrideOptions(
				_hotOptions,
				fakeUpdateLoaderOptions
			),
			compilerFactory: () => require("@rspack/core"),
			getBundle: RspackHotProcessor.findBundle(_hotOptions),
			compilerOptions: context =>
				readConfigFile<ECompilerType.Rspack>([
					context.getSource("rspack.config.js"),
					context.getSource("webpack.config.js")
				])[0],
			name: _hotOptions.name,
			testConfig: {
				timeout: 10000
			}
		});
		this.updateOptions = fakeUpdateLoaderOptions;
	}

	protected createRunner(
		env: ITestEnv,
		context: ITestContext,
		hotOptions: TCompilerOptions<ECompilerType.Rspack>
	): ITestRunner | null {
		if (this.runner) return this.runner;
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		this.runner = new HotRunner({
			env,
			stats: stats!,
			name: this._options.name,
			runInNewContext: false,
			testConfig: this._options.testConfig,
			source: context.getSource(),
			dist: context.getDist(),
			compilerOptions: hotOptions,
			next: (
				callback: (error: Error | null, stats?: StatsCompilation) => void
			) => {
				this.updateOptions.updateIndex++;
				compiler
					.build()
					.then(stats => {
						if (!stats)
							return callback(new Error("Should generate stats during build"));
						const jsonStats = stats.toJson({
							// errorDetails: true
						});
						if (
							checkArrayExpectation(
								context.getSource(),
								jsonStats,
								"error",
								"errors" + this.updateOptions.updateIndex,
								"Error",
								callback
							)
						) {
							return;
						}
						if (
							checkArrayExpectation(
								context.getSource(),
								jsonStats,
								"warning",
								"warnings" + this.updateOptions.updateIndex,
								"Warning",
								callback
							)
						) {
							return;
						}
						callback(null, jsonStats);
					})
					.catch(callback);
			}
		});
		return this.runner;
	}

	static findBundle(
		hotOptions: IRspackHotProcessorOptions
	): IBasicProcessorOptions<ECompilerType.Rspack>["getBundle"] {
		return context => {
			let files: string[] = [];
			let prefiles: string[] = [];
			const compiler = context.getCompiler(hotOptions.name);
			if (!compiler) throw new Error("Compiler should exists when find bundle");
			const stats = compiler.getStats();
			if (!stats) throw new Error("Stats should exists when find bundle");
			const info = stats.toJson({ all: false, entrypoints: true });
			if (hotOptions.target === "web" || hotOptions.target === "webworker") {
				for (const file of info.entrypoints!.main.assets!) {
					if (file.name.endsWith(".js")) {
						files.push(file.name);
					} else {
						prefiles.push(file.name);
					}
				}
			} else {
				const assets = info.entrypoints!.main.assets!.filter(s =>
					s.name.endsWith(".js")
				);
				files.push(assets[assets.length - 1].name);
			}
			return [...prefiles, ...files];
		};
	}

	static defaultOptions(
		hotOptions: IRspackHotProcessorOptions,
		updateOptions: TUpdateOptions
	): IBasicProcessorOptions<ECompilerType.Rspack>["defaultOptions"] {
		return (context: ITestContext) => ({
			context: context.getSource(),
			mode: "development",
			devtool: false,
			output: {
				path: context.getDist(),
				filename: "bundle.js",
				chunkFilename: "[name].chunk.[fullhash].js",
				publicPath: "https://test.cases/path/",
				library: { type: "commonjs2" }
			},
			optimization: {
				moduleIds: "named"
			},
			target: hotOptions.target,
			plugins: [new rspack.HotModuleReplacementPlugin()]
		});
	}
	static overrideOptions(
		hotOptions: IRspackHotProcessorOptions,
		updateOptions: TUpdateOptions
	): IBasicProcessorOptions<ECompilerType.Rspack>["overrideOptions"] {
		return (
			context: ITestContext,
			options: TCompilerOptions<ECompilerType.Rspack>
		) => {
			if (!options.entry) {
				options.entry = "./index.js";
			}
			options.module ??= {};
			options.module.rules ??= [];
			options.module.rules.push({
				test: /\.(js|css|json)/,
				use: [
					{
						loader: path.resolve(
							__dirname,
							"../helper/legacy/fake-update-loader.js"
						),
						options: updateOptions
					}
				]
			});
			options.plugins ??= [];
			options.plugins.push(new rspack.LoaderOptionsPlugin(updateOptions));
		};
	}
}
