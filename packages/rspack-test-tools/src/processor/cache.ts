import path from "node:path";
import { rspack } from "@rspack/core";
import { removeSync } from "fs-extra";

import { TestHotUpdatePlugin } from "../helper/plugins";
import {
	ECompilerType,
	type ITestContext,
	type ITestEnv,
	type ITestRunner,
	type TCompilerOptions,
	type TUpdateOptions
} from "../type";
import { BasicProcessor, type IBasicProcessorOptions } from "./basic";

export interface ICacheProcessorOptions<T extends ECompilerType>
	extends Omit<IBasicProcessorOptions<T>, "runable"> {
	target: TCompilerOptions<T>["target"];
}

export class CacheProcessor<T extends ECompilerType> extends BasicProcessor<T> {
	protected updateOptions: TUpdateOptions;
	protected runner: ITestRunner | null = null;

	constructor(protected _cacheOptions: ICacheProcessorOptions<T>) {
		const fakeUpdateLoaderOptions: TUpdateOptions = {
			updateIndex: 0,
			totalUpdates: 1,
			changedFiles: []
		};
		super({
			defaultOptions: CacheProcessor.defaultOptions,
			overrideOptions: CacheProcessor.overrideOptions,
			findBundle: CacheProcessor.findBundle,
			runable: true,
			..._cacheOptions
		});
		this.updateOptions = fakeUpdateLoaderOptions;
	}

	async build(context: ITestContext): Promise<void> {
		// clear cache directory first time.
		const experiments =
			this.getCompiler(context).getOptions().experiments || {};
		let directory = "";
		if (
			"cache" in experiments &&
			typeof experiments.cache === "object" &&
			experiments.cache.type === "persistent"
		) {
			directory = experiments.cache.storage?.directory || directory;
		}
		removeSync(
			path.resolve(context.getSource(), directory || "node_modules/.cache")
		);

		await super.build(context);
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
		this: CacheProcessor<T>,
		context: ITestContext
	): string[] {
		const files: string[] = [];
		const prefiles: string[] = [];
		const compiler = context.getCompiler(this._cacheOptions.name);
		if (!compiler) throw new Error("Compiler should exists when find bundle");
		const stats = compiler.getStats();
		if (!stats) throw new Error("Stats should exists when find bundle");
		const info = stats.toJson({ all: false, entrypoints: true });
		if (
			this._cacheOptions.target === "web" ||
			this._cacheOptions.target === "webworker"
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
		this: CacheProcessor<T>,
		context: ITestContext
	): TCompilerOptions<T> {
		const options = {
			context: context.getSource(),
			mode: "production",
			cache: true,
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
			target: this._cacheOptions.target,
			experiments: {
				css: true,
				rspackFuture: {
					bundlerInfo: {
						force: false
					}
				}
			}
		} as TCompilerOptions<T>;

		if (this._cacheOptions.compilerType === ECompilerType.Rspack) {
			options.plugins ??= [];
			(options as TCompilerOptions<ECompilerType.Rspack>).plugins!.push(
				new rspack.HotModuleReplacementPlugin()
			);
		}

		return options;
	}

	static overrideOptions<T extends ECompilerType>(
		this: CacheProcessor<T>,
		context: ITestContext,
		options: TCompilerOptions<T>
	): void {
		if (!options.entry) {
			options.entry = "./index.js";
		}

		options.module ??= {};
		for (const cssModuleType of ["css/auto", "css/module", "css"] as const) {
			options.module!.generator ??= {};
			options.module!.generator[cssModuleType] ??= {};
			options.module!.generator[cssModuleType]!.exportsOnly ??=
				this._cacheOptions.target === "async-node";
		}
		options.module.rules ??= [];
		options.module.rules.push({
			test: /\.(js|css|json)/,
			use: [
				{
					loader: path.resolve(__dirname, "../helper/loaders/hot-update.js"),
					options: this.updateOptions
				}
			]
		});
		if (this._cacheOptions.compilerType === ECompilerType.Rspack) {
			options.plugins ??= [];
			(options as TCompilerOptions<ECompilerType.Rspack>).plugins!.push(
				new TestHotUpdatePlugin(this.updateOptions)
			);
		}
	}
}
