import path from "path";
import { rspack } from "@rspack/core";

import {
	ECompilerType,
	type ITestContext,
	type ITestEnv,
	type ITestRunner,
	type TCompilerOptions,
	type TUpdateOptions
} from "../type";
import { BasicProcessor, type IBasicProcessorOptions } from "./basic";

export interface IHotProcessorOptions<T extends ECompilerType>
	extends Omit<IBasicProcessorOptions<T>, "runable"> {
	target: TCompilerOptions<T>["target"];
}

export class HotProcessor<T extends ECompilerType> extends BasicProcessor<T> {
	protected updateOptions: TUpdateOptions;
	protected runner: ITestRunner | null = null;

	constructor(protected _hotOptions: IHotProcessorOptions<T>) {
		const fakeUpdateLoaderOptions: TUpdateOptions = {
			updateIndex: 0,
			totalUpdates: 1,
			changedFiles: []
		};
		super({
			defaultOptions: HotProcessor.defaultOptions,
			overrideOptions: HotProcessor.overrideOptions,
			findBundle: HotProcessor.findBundle,
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
		this: HotProcessor<T>,
		context: ITestContext
	): string[] {
		let files: string[] = [];
		let prefiles: string[] = [];
		const compiler = context.getCompiler(this._hotOptions.name);
		if (!compiler) throw new Error("Compiler should exists when find bundle");
		const stats = compiler.getStats();
		if (!stats) throw new Error("Stats should exists when find bundle");
		const info = stats.toJson({ all: false, entrypoints: true });
		if (
			this._hotOptions.target === "web" ||
			this._hotOptions.target === "webworker"
		) {
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
	}

	async afterAll(context: ITestContext) {
		await super.afterAll(context);
		if (
			this.updateOptions.updateIndex + 1 !==
			this.updateOptions.totalUpdates
		) {
			throw new Error(
				`Should run all hot steps (${this.updateOptions.updateIndex + 1} / ${this.updateOptions.totalUpdates}): ${this._options.name}`
			);
		}
	}

	static defaultOptions<T extends ECompilerType>(
		this: HotProcessor<T>,
		context: ITestContext
	): TCompilerOptions<T> {
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
			target: this._hotOptions.target,
			experiments: {
				css: true,
				rspackFuture: {
					bundlerInfo: {
						force: false
					}
				}
			}
		} as TCompilerOptions<T>;

		if (this._hotOptions.compilerType === ECompilerType.Rspack) {
			options.plugins ??= [];
			(options as TCompilerOptions<ECompilerType.Rspack>).plugins!.push(
				new rspack.HotModuleReplacementPlugin()
			);
		}
		return options;
	}

	static overrideOptions<T extends ECompilerType>(
		this: HotProcessor<T>,
		context: ITestContext,
		options: TCompilerOptions<T>
	): void {
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
					options: this.updateOptions
				}
			]
		});
		if (this._hotOptions.compilerType === ECompilerType.Rspack) {
			options.plugins ??= [];
			(options as TCompilerOptions<ECompilerType.Rspack>).plugins!.push(
				new rspack.LoaderOptionsPlugin(this.updateOptions)
			);
		}
	}
}
