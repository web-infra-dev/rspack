import path from "node:path";
import { rspack } from "@rspack/core";

import { HotUpdatePlugin } from "../helper/hot-update";
import {
	ECompilerType,
	type ITestContext,
	type ITestEnv,
	type ITestRunner,
	type TCompilerOptions
} from "../type";
import { BasicProcessor, type IBasicProcessorOptions } from "./basic";

export interface ICacheProcessorOptions<T extends ECompilerType>
	extends Omit<IBasicProcessorOptions<T>, "runable"> {
	target: TCompilerOptions<T>["target"];
	tempDir: string;
	sourceDir: string;
}

export class CacheProcessor<T extends ECompilerType> extends BasicProcessor<T> {
	protected updatePlugin: HotUpdatePlugin;
	protected runner: ITestRunner | null = null;

	constructor(protected _cacheOptions: ICacheProcessorOptions<T>) {
		super({
			defaultOptions: CacheProcessor.defaultOptions,
			overrideOptions: CacheProcessor.overrideOptions,
			findBundle: CacheProcessor.findBundle,
			runable: true,
			..._cacheOptions
		});
		this.updatePlugin = new HotUpdatePlugin(
			_cacheOptions.sourceDir,
			_cacheOptions.tempDir
		);
	}

	async config(context: ITestContext) {
		await this.updatePlugin.initialize();
		this._options.configFiles = this._options.configFiles?.map(item => {
			return path.resolve(this._cacheOptions.tempDir, item);
		});
		super.config(context);
	}

	async run(env: ITestEnv, context: ITestContext) {
		context.setValue(this._options.name, "hotUpdateContext", this.updatePlugin);
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
		const updateIndex = this.updatePlugin.getUpdateIndex();
		const totalUpdates = this.updatePlugin.getTotalUpdates();
		if (updateIndex + 1 !== totalUpdates) {
			throw new Error(
				`Should run all hot steps (${updateIndex + 1} / ${totalUpdates}): ${this._options.name}`
			);
		}
	}

	static defaultOptions<T extends ECompilerType>(
		this: CacheProcessor<T>,
		context: ITestContext
	): TCompilerOptions<T> {
		const options = {
			context: this._cacheOptions.tempDir,
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
				moduleIds: "named",
				emitOnErrors: true
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

		// rewrite context to temp dir
		options.context = this._cacheOptions.tempDir;
		options.module ??= {};
		for (const cssModuleType of ["css/auto", "css/module", "css"] as const) {
			options.module!.generator ??= {};
			options.module!.generator[cssModuleType] ??= {};
			options.module!.generator[cssModuleType]!.exportsOnly ??=
				this._cacheOptions.target === "async-node";
		}
		if (this._cacheOptions.compilerType === ECompilerType.Rspack) {
			options.plugins ??= [];
			(options as TCompilerOptions<ECompilerType.Rspack>).plugins!.push(
				this.updatePlugin
			);
		}
		if (!global.printLogger) {
			options.infrastructureLogging = {
				level: "error"
			};
		}
	}
}
