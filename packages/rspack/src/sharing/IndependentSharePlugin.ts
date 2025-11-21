import { basename, join, resolve } from "node:path";

import type { Compiler } from "../Compiler";
import type { LibraryOptions, Plugins, RspackOptions } from "../config";
import {
	getFileName,
	type ModuleFederationManifestPluginOptions
} from "../container/ModuleFederationManifestPlugin";
import { parseOptions } from "../container/options";
import {
	CollectShareEntryPlugin,
	type ShareRequestsMap
} from "./CollectShareEntryPlugin";
import { ConsumeSharedPlugin } from "./ConsumeSharedPlugin";
import { OptimizeDependencyReferencedExportsPlugin } from "./OptimizeDependencyReferencedExportsPlugin";
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
	const pluginName = plugin.name || plugin.constructor?.name;
	if (!pluginName) {
		return true;
	}
	return ![
		"TreeshakeSharePlugin",
		"IndependentSharePlugin",
		"ModuleFederationPlugin",
		"OptimizeDependencyReferencedExportsPlugin",
		"HtmlWebpackPlugin"
	].includes(pluginName);
};

export interface IndependentSharePluginOptions {
	name: string;
	shared: Shared;
	library?: LibraryOptions;
	outputDir?: string;
	outputFilePath?: string;
	plugins?: Plugins;
	treeshake?: boolean;
	manifest?: ModuleFederationManifestPluginOptions;
	injectUsedExports?: boolean;
}

// { react: [  [ react/19.0.0/index.js , 19.0.0, react_global_name ]  ] }
export type ShareFallback = Record<string, [string, string, string][]>;

class VirtualEntryPlugin {
	sharedOptions: [string, SharedConfig][];
	constructor(sharedOptions: [string, SharedConfig][]) {
		this.sharedOptions = sharedOptions;
	}
	createEntry() {
		const { sharedOptions } = this;
		const entryContent = sharedOptions.reduce<string>((acc, cur, index) => {
			return `${acc}import shared_${index} from '${cur[0]}';\n`;
		}, "");
		return entryContent;
	}

	static entry() {
		return {
			[VIRTUAL_ENTRY_NAME]: VIRTUAL_ENTRY
		};
	}

	apply(compiler: Compiler) {
		new compiler.rspack.experiments.VirtualModulesPlugin({
			[VIRTUAL_ENTRY]: this.createEntry()
		}).apply(compiler);

		compiler.hooks.thisCompilation.tap(
			"RemoveVirtualEntryAsset",
			compilation => {
				compilation.hooks.processAssets.tapPromise(
					{
						name: "RemoveVirtualEntryAsset",
						stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE
					},
					async () => {
						try {
							const chunk = compilation.namedChunks.get(VIRTUAL_ENTRY_NAME);

							chunk?.files.forEach(f => {
								compilation.deleteAsset(f);
							});
						} catch (_e) {
							console.error("Failed to remove virtual entry file!");
						}
					}
				);
			}
		);
	}
}

export class IndependentSharePlugin {
	mfName: string;
	shared: Shared;
	library?: LibraryOptions;
	sharedOptions: [string, SharedConfig][];
	outputDir: string;
	outputFilePath?: string;
	plugins: Plugins;
	compilers: Map<string, Compiler> = new Map();
	treeshake?: boolean;
	manifest?: ModuleFederationManifestPluginOptions;
	buildAssets: ShareFallback = {};
	injectUsedExports?: boolean;

	name = "IndependentSharePlugin";
	constructor(options: IndependentSharePluginOptions) {
		const {
			outputDir,
			outputFilePath,
			plugins,
			treeshake,
			shared,
			name,
			manifest,
			injectUsedExports,
			library
		} = options;
		this.shared = shared;
		this.mfName = name;
		this.outputDir = outputFilePath ? "" : outputDir || "independent-packages";
		this.outputFilePath = outputFilePath;
		this.plugins = plugins || [];
		this.treeshake = treeshake;
		this.manifest = manifest;
		this.injectUsedExports = injectUsedExports ?? true;
		this.library = library;
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

	apply(compiler: Compiler) {
		compiler.hooks.beforeRun.tapAsync(
			"IndependentSharePlugin",
			(compiler, callback) => {
				void this.createIndependentCompilers(compiler)
					.then(() => callback())
					.catch(callback);
			}
		);

		// clean hooks
		compiler.hooks.shutdown.tapAsync("IndependentSharePlugin", callback => {
			this.cleanup();
			console.log("cleanup");
			callback();
		});

		// inject buildAssets to stats
		compiler.hooks.compilation.tap("IndependentSharePlugin", compilation => {
			compilation.hooks.processAssets.tapPromise(
				{
					name: "injectBuildAssets",
					stage: (compilation.constructor as any)
						.PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER
				},
				async () => {
					if (!this.manifest) {
						return;
					}
					const { statsFileName } = getFileName(this.manifest);
					const stats = compilation.getAsset(statsFileName);
					if (!stats) {
						return;
					}
					const statsContent = JSON.parse(stats.source.source().toString()) as {
						shared: {
							name: string;
							version: string;
							fallback?: string;
							fallbackName?: string;
						}[];
					};

					const { shared } = statsContent;
					Object.entries(this.buildAssets).forEach(([key, item]) => {
						const targetShared = shared.find(s => s.name === key);
						if (!targetShared) {
							return;
						}
						item.forEach(([entry, version, globalName]) => {
							if (version === targetShared.version) {
								targetShared.fallback = entry;
								targetShared.fallbackName = globalName;
							}
						});
					});

					compilation.updateAsset(
						statsFileName,
						new compiler.webpack.sources.RawSource(JSON.stringify(statsContent))
					);
				}
			);
		});
	}

	private async createIndependentCompilers(parentCompiler: Compiler) {
		const { sharedOptions, buildAssets } = this;
		console.log("🚀 Start creating a standalone compiler...");

		const parentOutputDir = parentCompiler.options.output.path
			? basename(parentCompiler.options.output.path)
			: "";
		// collect share requests for each shareName and then build share container
		const shareRequestsMap: ShareRequestsMap =
			await this.createIndependentCompiler(parentCompiler, parentOutputDir);

		await Promise.all(
			sharedOptions.map(async ([shareName, shareConfig]) => {
				if (!shareConfig.treeshake || shareConfig.import === false) {
					return;
				}
				const shareRequests = shareRequestsMap[shareName].requests;
				await Promise.all(
					shareRequests.map(async ([request, version]) => {
						const sharedConfig = sharedOptions.find(
							([name]) => name === shareName
						)?.[1];
						const [shareFileName, globalName] =
							await this.createIndependentCompiler(
								parentCompiler,
								parentOutputDir,
								{
									shareRequestsMap,
									currentShare: {
										shareName,
										version,
										request,
										independentShareFileName:
											sharedConfig?.independentShareFileName
									}
								}
							);
						if (typeof shareFileName === "string") {
							buildAssets[shareName] ||= [];
							buildAssets[shareName].push([shareFileName, version, globalName]);
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
		const {
			mfName,
			plugins,
			outputDir,
			outputFilePath,
			sharedOptions,
			treeshake,
			library
		} = this;
		const outputDirWithShareName =
			outputFilePath ||
			join(outputDir, encodeName(extraOptions?.currentShare?.shareName || ""));

		const parentConfig = parentCompiler.options;

		const finalPlugins = [];
		const rspack = parentCompiler.rspack;
		let extraPlugin: CollectShareEntryPlugin | ShareContainerPlugin;
		if (!extraOptions) {
			extraPlugin = new CollectShareEntryPlugin({
				sharedOptions,
				shareScope: "default"
			});
		} else {
			extraPlugin = new ShareContainerPlugin({
				mfName,
				library,
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
		finalPlugins.push(extraPlugin);

		finalPlugins.push(
			new ConsumeSharedPlugin({
				consumes: sharedOptions
					.filter(
						([key, options]) =>
							extraOptions?.currentShare.shareName !== (options.shareKey || key)
					)
					.map(([key, options]) => ({
						[key]: {
							import: false,
							shareKey: options.shareKey || key,
							shareScope: options.shareScope,
							requiredVersion: options.requiredVersion,
							strictVersion: options.strictVersion,
							singleton: options.singleton,
							packageName: options.packageName,
							eager: options.eager
						}
					})),
				enhanced: true
			})
		);

		if (treeshake) {
			finalPlugins.push(
				new OptimizeDependencyReferencedExportsPlugin(
					sharedOptions,
					this.injectUsedExports
				)
			);
		}
		finalPlugins.push(
			new VirtualEntryPlugin(sharedOptions)
			// new rspack.experiments.VirtualModulesPlugin({
			// 	[VIRTUAL_ENTRY]: this.createEntry()
			// })
		);
		const fullOutputDir = resolve(
			parentCompiler.outputPath,
			outputDirWithShareName
		);
		const compilerConfig: RspackOptions = {
			...parentConfig,
			mode: parentConfig.mode || "development",

			entry: VirtualEntryPlugin.entry,

			output: {
				path: fullOutputDir,
				clean: true,
				publicPath: parentConfig.output?.publicPath || "auto"
			},

			plugins: finalPlugins,

			optimization: {
				...parentConfig.optimization,
				splitChunks: false
			}
		};

		const compiler = rspack.rspack(compilerConfig);

		compiler.inputFileSystem = parentCompiler.inputFileSystem;
		compiler.outputFileSystem = parentCompiler.outputFileSystem;
		compiler.intermediateFileSystem = parentCompiler.intermediateFileSystem;

		const { currentShare } = extraOptions || {};
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
					console.log(`✅ 独立包 ${currentShare.shareName} 编译成功`);

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
