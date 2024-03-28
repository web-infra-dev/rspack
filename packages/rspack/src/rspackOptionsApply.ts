/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/OptionsApply.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import {
	RspackOptionsNormalized,
	Compiler,
	OptimizationRuntimeChunkNormalized,
	RspackPluginFunction
} from ".";
import fs from "graceful-fs";

import { DefaultStatsFactoryPlugin } from "./stats/DefaultStatsFactoryPlugin";
import { DefaultStatsPrinterPlugin } from "./stats/DefaultStatsPrinterPlugin";
import { cleverMerge } from "./util/cleverMerge";
import assert from "assert";
import IgnoreWarningsPlugin from "./lib/ignoreWarningsPlugin";
import EntryOptionPlugin from "./lib/EntryOptionPlugin";
import {
	ArrayPushCallbackChunkFormatPlugin,
	CommonJsChunkFormatPlugin,
	ElectronTargetPlugin,
	EnableChunkLoadingPlugin,
	EnableLibraryPlugin,
	EnableWasmLoadingPlugin,
	ExternalsPlugin,
	HttpExternalsRspackPlugin,
	ModuleChunkFormatPlugin,
	NodeTargetPlugin,
	DefinePlugin,
	MergeDuplicateChunksPlugin,
	SplitChunksPlugin,
	ChunkPrefetchPreloadPlugin,
	NamedModuleIdsPlugin,
	DeterministicModuleIdsPlugin,
	NamedChunkIdsPlugin,
	DeterministicChunkIdsPlugin,
	RealContentHashPlugin,
	RemoveEmptyChunksPlugin,
	EnsureChunkConditionsPlugin,
	WarnCaseSensitiveModulesPlugin,
	DataUriPlugin,
	FileUriPlugin,
	JavascriptModulesPlugin,
	JsonModulesPlugin,
	AsyncWebAssemblyModulesPlugin,
	RuntimePlugin,
	InferAsyncModulesPlugin,
	WorkerPlugin,
	EvalSourceMapDevToolPlugin,
	SourceMapDevToolPlugin,
	AssetModulesPlugin,
	MangleExportsPlugin,
	FlagDependencyExportsPlugin,
	FlagDependencyUsagePlugin,
	SideEffectsFlagPlugin,
	BundlerInfoRspackPlugin,
	ModuleConcatenationPlugin,
	EvalDevToolModulePlugin,
	JsLoaderRspackPlugin,
	LazyCompilationPlugin
} from "./builtin-plugin";
import { deprecatedWarn } from "./util";

export function applyEntryOptions(
	compiler: Compiler,
	options: RspackOptionsNormalized
) {
	if (!options.experiments.rspackFuture!.disableApplyEntryLazily) {
		deprecatedWarn(
			`You are depending on apply entry lazily (https://rspack.dev/config/experiments.html#experimentsrspackfuturedisableapplyentrylazily), this behavior has been deprecated, you can setup 'experiments.rspackFuture.disableApplyEntryLazily = true' to disable this behavior, and this will be enabled by default in v0.5`
		);
	}
	if (compiler.parentCompilation === undefined) {
		if (options.experiments.rspackFuture!.disableApplyEntryLazily) {
			new EntryOptionPlugin().apply(compiler);
		} else {
			EntryOptionPlugin.applyEntryOption(
				compiler,
				compiler.context,
				options.entry
			);
		}
	}
}

export class RspackOptionsApply {
	constructor() {}
	process(options: RspackOptionsNormalized, compiler: Compiler) {
		assert(
			options.output.path,
			"options.output.path should have value after `applyRspackOptionsDefaults`"
		);
		compiler.outputPath = options.output.path;
		compiler.name = options.name;
		compiler.outputFileSystem = fs;

		if (options.externals) {
			assert(
				options.externalsType,
				"options.externalsType should have value after `applyRspackOptionsDefaults`"
			);
			new ExternalsPlugin(options.externalsType, options.externals).apply(
				compiler
			);
		}

		if (options.externalsPresets.node) {
			new NodeTargetPlugin().apply(compiler);
		}
		if (options.externalsPresets.electronMain) {
			new ElectronTargetPlugin("main").apply(compiler);
		}
		if (options.externalsPresets.electronPreload) {
			new ElectronTargetPlugin("preload").apply(compiler);
		}
		if (options.externalsPresets.electronRenderer) {
			new ElectronTargetPlugin("renderer").apply(compiler);
		}
		if (
			options.externalsPresets.electron &&
			!options.externalsPresets.electronMain &&
			!options.externalsPresets.electronPreload &&
			!options.externalsPresets.electronRenderer
		) {
			new ElectronTargetPlugin().apply(compiler);
		}
		if (
			options.externalsPresets.web ||
			options.externalsPresets.webAsync ||
			(options.externalsPresets.node && options.experiments.css)
		) {
			new HttpExternalsRspackPlugin(
				!!options.experiments.css,
				!!options.externalsPresets.webAsync
			).apply(compiler);
		}

		new ChunkPrefetchPreloadPlugin().apply(compiler);

		if (typeof options.output.chunkFormat === "string") {
			switch (options.output.chunkFormat) {
				case "array-push": {
					new ArrayPushCallbackChunkFormatPlugin().apply(compiler);
					break;
				}
				case "commonjs": {
					new CommonJsChunkFormatPlugin().apply(compiler);
					break;
				}
				case "module": {
					new ModuleChunkFormatPlugin().apply(compiler);
					break;
				}
				default:
					throw new Error(
						"Unsupported chunk format '" + options.output.chunkFormat + "'."
					);
			}
		}

		if (
			options.output.enabledChunkLoadingTypes &&
			options.output.enabledChunkLoadingTypes.length > 0
		) {
			for (const type of options.output.enabledChunkLoadingTypes) {
				new EnableChunkLoadingPlugin(type).apply(compiler);
			}
		}

		if (
			options.output.enabledWasmLoadingTypes &&
			options.output.enabledWasmLoadingTypes.length > 0
		) {
			for (const type of options.output.enabledWasmLoadingTypes) {
				new EnableWasmLoadingPlugin(type).apply(compiler);
			}
		}

		const runtimeChunk = options.optimization
			.runtimeChunk as OptimizationRuntimeChunkNormalized;
		if (runtimeChunk) {
			Object.entries(options.entry).forEach(([entryName, value]) => {
				if (value.runtime === undefined) {
					value.runtime = runtimeChunk.name({ name: entryName });
				}
			});
		}

		if (options.devtool) {
			if (options.devtool.includes("source-map")) {
				const hidden = options.devtool.includes("hidden");
				const inline = options.devtool.includes("inline");
				const evalWrapped = options.devtool.includes("eval");
				const cheap = options.devtool.includes("cheap");
				const moduleMaps = options.devtool.includes("module");
				const noSources = options.devtool.includes("nosources");
				const Plugin = evalWrapped
					? EvalSourceMapDevToolPlugin
					: SourceMapDevToolPlugin;
				new Plugin({
					filename: inline ? null : options.output.sourceMapFilename,
					moduleFilenameTemplate: options.output.devtoolModuleFilenameTemplate,
					fallbackModuleFilenameTemplate:
						options.output.devtoolFallbackModuleFilenameTemplate,
					append: hidden ? false : undefined,
					module: moduleMaps ? true : cheap ? false : true,
					columns: cheap ? false : true,
					noSources: noSources,
					namespace: options.output.devtoolNamespace
				}).apply(compiler);
			} else if (options.devtool.includes("eval")) {
				new EvalDevToolModulePlugin({
					moduleFilenameTemplate: options.output.devtoolModuleFilenameTemplate,
					namespace: options.output.devtoolNamespace
				}).apply(compiler);
			}
		}

		new JavascriptModulesPlugin().apply(compiler);
		new JsonModulesPlugin().apply(compiler);
		new AssetModulesPlugin().apply(compiler);
		if (options.experiments.asyncWebAssembly) {
			new AsyncWebAssemblyModulesPlugin().apply(compiler);
		}

		if (options.experiments.rspackFuture!.disableApplyEntryLazily) {
			applyEntryOptions(compiler, options);
		}
		assert(
			options.context,
			"options.context should have value after `applyRspackOptionsDefaults`"
		);
		compiler.hooks.entryOption.call(options.context, options.entry);

		new RuntimePlugin().apply(compiler);

		if (options.experiments.rspackFuture!.bundlerInfo) {
			new BundlerInfoRspackPlugin(
				options.experiments.rspackFuture!.bundlerInfo
			).apply(compiler);
		}

		new InferAsyncModulesPlugin().apply(compiler);

		new DataUriPlugin().apply(compiler);
		new FileUriPlugin().apply(compiler);

		new EnsureChunkConditionsPlugin().apply(compiler);
		if (options.optimization.mergeDuplicateChunks) {
			new MergeDuplicateChunksPlugin().apply(compiler);
		}

		if (options.experiments.rspackFuture?.newTreeshaking) {
			if (options.optimization.sideEffects) {
				new SideEffectsFlagPlugin(/* options.optimization.sideEffects === true */).apply(
					compiler
				);
			}
			if (options.optimization.providedExports) {
				new FlagDependencyExportsPlugin().apply(compiler);
			}
			if (options.optimization.usedExports) {
				new FlagDependencyUsagePlugin(
					options.optimization.usedExports === "global"
				).apply(compiler);
			}
			if (options.optimization.concatenateModules) {
				new ModuleConcatenationPlugin().apply(compiler);
			}
			if (options.optimization.mangleExports) {
				new MangleExportsPlugin(
					options.optimization.mangleExports !== "size"
				).apply(compiler);
			}
		}

		if (options.experiments.lazyCompilation) {
			const lazyOptions = options.experiments.lazyCompilation;

			new LazyCompilationPlugin(
				// this is only for test
				lazyOptions.cacheable ?? true,
				lazyOptions.entries ?? true,
				lazyOptions.imports ?? true,
				lazyOptions.test
					? {
							source: lazyOptions.test.source,
							flags: lazyOptions.test.flags
						}
					: undefined,
				lazyOptions.backend
			).apply(compiler);
		}

		if (
			options.output.enabledLibraryTypes &&
			options.output.enabledLibraryTypes.length > 0
		) {
			for (const type of options.output.enabledLibraryTypes) {
				new EnableLibraryPlugin(type).apply(compiler);
			}
		}
		if (options.optimization.splitChunks) {
			new SplitChunksPlugin(options.optimization.splitChunks).apply(compiler);
		}
		// TODO: inconsistent: the plugin need to be placed after SplitChunksPlugin
		if (options.optimization.removeEmptyChunks) {
			new RemoveEmptyChunksPlugin().apply(compiler);
		}
		if (options.optimization.realContentHash) {
			new RealContentHashPlugin().apply(compiler);
		}
		const moduleIds = options.optimization.moduleIds;
		if (moduleIds) {
			switch (moduleIds) {
				case "named": {
					new NamedModuleIdsPlugin().apply(compiler);
					break;
				}
				case "deterministic": {
					new DeterministicModuleIdsPlugin().apply(compiler);
					break;
				}
				default:
					throw new Error(`moduleIds: ${moduleIds} is not implemented`);
			}
		}
		const chunkIds = options.optimization.chunkIds;
		if (chunkIds) {
			switch (chunkIds) {
				case "named": {
					new NamedChunkIdsPlugin().apply(compiler);
					break;
				}
				case "deterministic": {
					new DeterministicChunkIdsPlugin().apply(compiler);
					break;
				}
				default:
					throw new Error(`chunkIds: ${chunkIds} is not implemented`);
			}
		}
		if (options.optimization.nodeEnv) {
			new DefinePlugin({
				"process.env.NODE_ENV": JSON.stringify(options.optimization.nodeEnv)
			}).apply(compiler);
		}
		const { minimize, minimizer } = options.optimization;
		if (minimize && minimizer) {
			for (const item of minimizer) {
				if (typeof item === "function") {
					(item as RspackPluginFunction).call(compiler, compiler);
				} else if (item !== "..." && item) {
					item.apply(compiler);
				}
			}
		}

		new WarnCaseSensitiveModulesPlugin().apply(compiler);

		new WorkerPlugin(
			options.output.workerChunkLoading!,
			options.output.workerWasmLoading!,
			options.output.module!,
			options.output.workerPublicPath!
		).apply(compiler);

		new DefaultStatsFactoryPlugin().apply(compiler);
		new DefaultStatsPrinterPlugin().apply(compiler);

		if (options.ignoreWarnings && options.ignoreWarnings.length > 0) {
			new IgnoreWarningsPlugin(options.ignoreWarnings).apply(compiler);
		}

		compiler.hooks.afterPlugins.call(compiler);
		if (!compiler.inputFileSystem) {
			throw new Error("No input filesystem provided");
		}
		compiler.resolverFactory.hooks.resolveOptions
			.for("normal")
			.tap("RspackOptionsApply", resolveOptions => {
				resolveOptions = cleverMerge(options.resolve, resolveOptions);
				resolveOptions.fileSystem = compiler.inputFileSystem;
				return resolveOptions;
			});
		compiler.resolverFactory.hooks.resolveOptions
			.for("context")
			.tap("RspackOptionsApply", resolveOptions => {
				resolveOptions = cleverMerge(options.resolve, resolveOptions);
				resolveOptions.fileSystem = compiler.inputFileSystem;
				resolveOptions.resolveToContext = true;
				return resolveOptions;
			});
		compiler.hooks.afterResolvers.call(compiler);
	}
}
