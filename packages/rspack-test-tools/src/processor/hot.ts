import { rspack } from "@rspack/core";
import path from "path";

import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestRunner,
	TCompilerOptions
} from "../type";
import { BasicTaskProcessor, IBasicProcessorOptions } from "./basic";

export interface IRspackHotProcessorOptions {
	name: string;
	target: TCompilerOptions<ECompilerType.Rspack>["target"];
}

export type TUpdateOptions = {
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
			compilerType: ECompilerType.Rspack,
			findBundle: RspackHotProcessor.findBundle(_hotOptions),
			configFiles: ["rspack.config.js", "webpack.config.js"],
			name: _hotOptions.name,
			runable: true
		});
		this.updateOptions = fakeUpdateLoaderOptions;
	}

	async run(env: ITestEnv, context: ITestContext) {
		context.setValue(
			this._options.name,
			"hotUpdateContext",
			this.updateOptions
		);
		await super.run(env, context);
	}

	static findBundle(
		hotOptions: IRspackHotProcessorOptions
	): IBasicProcessorOptions<ECompilerType.Rspack>["findBundle"] {
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
