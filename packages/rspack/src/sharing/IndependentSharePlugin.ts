import { basename, join, resolve } from "node:path";

import type { Compiler } from "../Compiler";
import type { Plugins, RspackOptions } from "../config";
import { parseOptions } from "../container/options";
import {
	CollectShareEntryPlugin,
	type ShareRequestsMap
} from "./CollectShareEntryPlugin";
import {
	ShareContainerPlugin,
	type ShareContainerPluginOptions
} from "./ShareContainerPlugin";
import type { Shared, SharedConfig } from "./SharePlugin";
import { encodeName, isRequiredVersion } from "./utils";

const VIRTUAL_ENTRY = "./virtual-entry.js";
const VIRTUAL_ENTRY_NAME = "virtual-entry";

export type MakeRequired<T, K extends keyof T> = Required<Pick<T, K>> &
	Omit<T, K>;

const filterPlugin = (plugin: Plugins[0]) => {
	if (!plugin) {
		return true;
	}
	const pluginName = plugin["name"] || plugin["constructor"]?.name;
	if (!pluginName) {
		return true;
	}
	return ![
		"IndependentSharePlugin",
		"ModuleFederationPlugin",
		"OptimizeDependencyReferencedExportsPlugin",
		"HtmlWebpackPlugin"
	].includes(pluginName);
};

export interface IndependentSharePluginOptions {
	name: string;
	shared: Shared;
	outputDir?: string;
	plugins?: Plugins;
	treeshake?: boolean;
}

export type ShareFallback = Record<string, [string, string][]>;

export class IndependentSharePlugin {
	mfName: string;
	shared: Shared;
	sharedOptions: [string, SharedConfig][];
	outputDir: string;
	plugins: Plugins;
	compilers: Map<string, Compiler> = new Map();
	treeshake?: boolean;
	// { react: [  [ react/19.0.0/index.js , 19.0.0 ]  ] }
	buildAssets: ShareFallback = {};

	name = "IndependentSharePlugin";
	constructor(options: IndependentSharePluginOptions) {
		const { outputDir, plugins, treeshake, shared, name } = options;
		if (!shared) {
			throw new Error("mfConfig.shared is required for share treeshake");
		}
		if (!name) {
			throw new Error("mfConfig.name is required for share treeshake");
		}
		this.shared = shared;
		this.mfName = name;
		this.outputDir = outputDir || "independent-packages";
		this.plugins = plugins || [];
		this.treeshake = treeshake;
		this.sharedOptions = parseOptions(
			shared,
			(item, key) => {
				if (typeof item !== "string")
					throw new Error(
						`Unexpected array in shared configuration for key "${key}"`
					);
				const config: SharedConfig =
					item === key || !isRequiredVersion(item)
						? {
								import: item
							}
						: {
								import: key,
								requiredVersion: item
							};

				return config;
			},
			item => {
				return item;
			}
		);
	}

	static IndependentShareBuildAssetsFilename =
		"independent-share-build-assets.json";

	apply(compiler: Compiler) {
		compiler.hooks.beforeRun.tapAsync(
			"IndependentSharePlugin",
			async (compiler, callback) => {
				// only call once
				await this.createIndependentCompilers(compiler);
				console.log("createIndependentCompilers done");
				callback();
			}
		);

		// clean hooks
		compiler.hooks.shutdown.tapAsync("IndependentSharePlugin", callback => {
			this.cleanup();
			console.log("cleanup");
			callback();
		});

		compiler.hooks.compilation.tap("IndependentSharePlugin", compilation => {
			// compilation.hooks.additionalTreeRuntimeRequirements.tap(
			// 	'OptimizeDependencyReferencedExportsPlugin',
			// 	(chunk) => {
			// 		compilation.addRuntimeModule(
			// 			chunk,
			// 			new IndependentShareRuntimeModule(this.buildAssets),
			// 		);
			// 	},
			// );

			// inject buildAssets to stats
			compilation.hooks.processAssets.tapPromise(
				{
					name: "injectReferenceExports",
					stage: (compilation.constructor as any)
						.PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER
				},
				async () => {
					// if treeshake is enabled, it means current is second-build -- re-shake assets, no need to modify stats.
					compilation.emitAsset(
						IndependentSharePlugin.IndependentShareBuildAssetsFilename,
						new compiler.webpack.sources.RawSource(
							JSON.stringify(this.buildAssets)
						)
					);
					return;

					// add until manifest merge
					// const stats = compilation.getAsset(StatsFileName);
					// if (!stats) {
					// 	return;
					// }
					// const statsContent = JSON.parse(
					// 	stats.source.source().toString(),
					// ) as Stats;

					// const { shared } = statsContent;
					// Object.entries(this.buildAssets).forEach(([key, [name, entry]]) => {
					// 	const targetShared = shared.find((s) => s.name === key);
					// 	if (!targetShared) {
					// 		return;
					// 	}
					// 	targetShared.fallback = entry;
					// 	targetShared.fallbackName = name;
					// });

					// compilation.updateAsset(
					// 	StatsFileName,
					// 	new compiler.webpack.sources.RawSource(
					// 		JSON.stringify(statsContent),
					// 	),
					// );
				}
			);
		});
	}

	private createEntry() {
		const { sharedOptions } = this;
		const entryContent = sharedOptions.reduce<string>((acc, cur, index) => {
			return `${acc}import shared_${index} from '${cur[0]}';\n`;
		}, "");
		return entryContent;
	}

	private async createIndependentCompilers(parentCompiler: Compiler) {
		const { sharedOptions, buildAssets } = this;
		console.log("🚀 Start creating a standalone compiler...");

		const parentOutputDir = parentCompiler.options.output.path
			? basename(parentCompiler.options.output.path)
			: "";
		const shareRequestsMap: ShareRequestsMap =
			await this.createIndependentCompiler(parentCompiler, parentOutputDir);

		await Promise.all(
			sharedOptions.map(async ([shareName, shareConfig]) => {
				if (!shareConfig.treeshake) {
					return;
				}
				const shareRequests = shareRequestsMap[shareName].requests;
				await Promise.all(
					shareRequests.map(async ([request, version]) => {
						const shareFileName = await this.createIndependentCompiler(
							parentCompiler,
							parentOutputDir,
							{
								shareRequestsMap,
								currentShare: {
									shareName,
									version,
									request
								}
							}
						);
						if (typeof shareFileName === "string") {
							buildAssets[shareName] ||= [];
							buildAssets[shareName].push([shareFileName, version]);
						}
					})
				);
			})
		);

		console.log("✅ All independent packages have been compiled successfully");
	}

	private async createIndependentCompiler(
		parentCompiler: Compiler,
		parentOutputDir: string,
		extraOptions?: {
			currentShare: Omit<ShareContainerPluginOptions, "mfName">;
			shareRequestsMap: ShareRequestsMap;
		}
	) {
		const { name, plugins, outputDir, sharedOptions } = this;
		const outputDirWithShareName = join(
			outputDir,
			encodeName(extraOptions?.currentShare?.shareName || "")
		);

		const parentConfig = parentCompiler.options;

		const finalPlugins = [];
		// 创建独立的  compiler 实例
		const rspack = parentCompiler.rspack;
		let extraPlugin: CollectShareEntryPlugin | ShareContainerPlugin;
		if (!extraOptions) {
			extraPlugin = new CollectShareEntryPlugin({
				sharedOptions,
				shareScope: "default"
			});
		} else {
			extraPlugin = new ShareContainerPlugin({
				mfName: name,
				...extraOptions.currentShare
			});
		}
		(parentConfig.plugins || []).forEach(plugin => {
			if (
				plugin !== undefined &&
				typeof plugin !== "string" &&
				filterPlugin(plugin)
			) {
				finalPlugins.push(plugin);
			}
		});
		plugins.forEach(plugin => {
			finalPlugins.push(plugin);
		});
		// finalPlugins.push(
		// 	new TreeshakeConsumeSharedPlugin({
		// 		consumes: Object.keys(mfConfig.shared as Record<string, any>).reduce(
		// 			(acc, cur) => {
		// 				if (cur !== currentShare) {
		// 					// @ts-ignore
		// 					acc[cur] = {
		// 						// use current host shared
		// 						import: false,
		// 					};
		// 				}
		// 				return acc;
		// 			},
		// 			{},
		// 		),
		// 	}),
		// );

		// if (treeshake) {
		// 	finalPlugins.push(
		// 		new OptimizeDependencyReferencedExportsPlugin(
		// 			parseOptions(
		// 				mfConfig.shared,
		// 				(item, key) => {
		// 					if (typeof item !== 'string')
		// 						throw new Error(
		// 							`Unexpected array in shared configuration for key "${key}"`,
		// 						);
		// 					const config: SharedConfig =
		// 						item === key || !isRequiredVersion(item)
		// 							? {
		// 								import: item,
		// 							}
		// 							: {
		// 								import: key,
		// 								requiredVersion: item,
		// 							};

		// 					return config;
		// 				},
		// 				(item) => item,
		// 			),
		// 			[IGNORED_ENTRY],
		// 		),
		// 	);
		// }
		finalPlugins.push(extraPlugin);
		finalPlugins.push(
			new rspack.experiments.VirtualModulesPlugin({
				[VIRTUAL_ENTRY]: this.createEntry()
			})
		);
		const fullOutputDir = resolve(
			parentCompiler.context,
			parentOutputDir,
			outputDirWithShareName
		);
		const compilerConfig: RspackOptions = {
			...parentConfig,
			mode: parentConfig.mode || "development",

			entry: {
				[VIRTUAL_ENTRY_NAME]: VIRTUAL_ENTRY
			},

			// 输出配置
			output: {
				path: fullOutputDir,
				// filename: output || `${name}.js`,
				// library: {
				//   type: 'umd',
				//   name: libraryName || `__${name.replace(/-/g, '_')}__`,
				// },
				clean: true,
				publicPath: parentConfig.output?.publicPath || "auto"
			},

			// 插件继承
			plugins: finalPlugins,

			// 优化配置继承
			optimization: {
				...parentConfig.optimization,
				splitChunks: false // 每个包独立，不拆分
			}
		};

		const compiler = rspack.rspack(compilerConfig);

		// 设置文件系统
		compiler.inputFileSystem = parentCompiler.inputFileSystem;
		compiler.outputFileSystem = parentCompiler.outputFileSystem;
		compiler.intermediateFileSystem = parentCompiler.intermediateFileSystem;

		const { currentShare } = extraOptions || {};
		// 存储编译器引用
		currentShare &&
			this.compilers.set(
				`${currentShare.shareName}@${currentShare.version}`,
				compiler
			);

		return new Promise<any>((resolve, reject) => {
			compiler.run((err: any, stats: any) => {
				if (err || stats?.hasErrors()) {
					const target = currentShare ? currentShare.shareName : "收集依赖";
					console.error(
						`❌ ${target} 编译失败:`,
						err ||
							stats
								.toJson()
								.errors.map((e: Error) => e.message)
								.join("\n")
					);
					reject(err || new Error(`${target} 编译失败`));
					return;
				}

				currentShare &&
					console.log(
						// `✅ 独立包 ${name} 编译成功: ${join(outputPath, output || `${name}.js`)}`,
						`✅ 独立包 ${currentShare.shareName} 编译成功`
					);

				if (stats) {
					currentShare && console.log(`📊 ${currentShare.shareName} 编译统计:`);
					console.log(
						stats.toString({
							colors: true,
							chunks: false,
							modules: false
						})
					);
				}

				resolve(extraPlugin.getData());
			});
		});
	}

	// 获取所有编译器的状态
	getCompilerStatus() {
		return Array.from(this.compilers.entries()).map(([name, compiler]) => ({
			name,
			running: compiler.running,
			watching: compiler.watching
		}));
	}

	// 获取编译结果信息
	getCompilationResults() {
		return Array.from(this.compilers.entries()).map(([name, compiler]) => ({
			name,
			outputPath: resolve(compiler.options.output?.path || ""),
			entry: compiler.options.entry,
			library: compiler.options.output?.library
		}));
	}

	// 清理所有编译器
	private cleanup() {
		this.compilers.forEach(compiler => {
			if (compiler.watching) {
				compiler.watching.close(() => {
					console.log("👋 编译器已关闭");
				});
			}
		});
		this.compilers.clear();
	}
}
