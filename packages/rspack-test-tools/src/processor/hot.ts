import { rspack } from "@rspack/core";
import path from "path";

import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestRunner,
	TCompilerOptions
} from "../type";
import { BasicProcessor, IBasicProcessorOptions } from "./basic";

export interface IHotProcessorOptions<T extends ECompilerType>
	extends Omit<IBasicProcessorOptions<T>, "runable"> {
	target: TCompilerOptions<T>["target"];
}

export type TUpdateOptions = {
	updateIndex: number;
};

export class HotProcessor<T extends ECompilerType> extends BasicProcessor<T> {
	protected updateOptions: TUpdateOptions;
	protected runner: ITestRunner | null = null;

	constructor(protected _hotOptions: IHotProcessorOptions<T>) {
		const fakeUpdateLoaderOptions: TUpdateOptions = {
			updateIndex: 0
		};
		super({
			defaultOptions: HotProcessor.defaultOptions(
				_hotOptions,
				fakeUpdateLoaderOptions
			),
			overrideOptions: HotProcessor.overrideOptions(
				_hotOptions,
				fakeUpdateLoaderOptions
			),
			findBundle: HotProcessor.findBundle(_hotOptions),
			runable: true,
			..._hotOptions
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

	static findBundle<T extends ECompilerType>(
		hotOptions: IHotProcessorOptions<T>
	): IBasicProcessorOptions<T>["findBundle"] {
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

	static defaultOptions<T extends ECompilerType>(
		hotOptions: IHotProcessorOptions<T>,
		updateOptions: TUpdateOptions
	): IBasicProcessorOptions<T>["defaultOptions"] {
		return (context: ITestContext) => {
			const options = {
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
				target: hotOptions.target
			} as TCompilerOptions<T>;

			if (hotOptions.compilerType === ECompilerType.Rspack) {
				options.plugins ??= [];
				(options as TCompilerOptions<ECompilerType.Rspack>).plugins!.push(
					new rspack.HotModuleReplacementPlugin()
				);
			}
			return options;
		};
	}

	static overrideOptions<T extends ECompilerType>(
		hotOptions: IHotProcessorOptions<T>,
		updateOptions: TUpdateOptions
	): IBasicProcessorOptions<T>["overrideOptions"] {
		return (context: ITestContext, options: TCompilerOptions<T>) => {
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
			if (hotOptions.compilerType === ECompilerType.Rspack) {
				options.plugins ??= [];
				(options as TCompilerOptions<ECompilerType.Rspack>).plugins!.push(
					new rspack.LoaderOptionsPlugin(updateOptions)
				);
			}
		};
	}
}
