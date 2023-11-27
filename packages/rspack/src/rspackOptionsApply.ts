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

import { ResolveSwcPlugin } from "./web/ResolveSwcPlugin";
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
	OldSplitChunksPlugin
} from "./builtin-plugin";
import { ModuleFederationRuntimePlugin } from "./container/ModuleFederationRuntimePlugin";

export function optionsApply_compat(
	compiler: Compiler,
	options: RspackOptionsNormalized
) {
	if (compiler.parentCompilation === undefined) {
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

		if (
			options.output.enabledLibraryTypes &&
			options.output.enabledLibraryTypes.length > 0
		) {
			for (const type of options.output.enabledLibraryTypes) {
				new EnableLibraryPlugin(type).apply(compiler);
			}
		}

		// TODO: change to new EntryOptionPlugin().apply(compiler);
		EntryOptionPlugin.applyEntryOption(
			compiler,
			compiler.context,
			options.entry
		);

		if (options.devServer?.hot) {
			new compiler.webpack.HotModuleReplacementPlugin().apply(compiler);
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

		const runtimeChunk = options.optimization
			.runtimeChunk as OptimizationRuntimeChunkNormalized;
		if (runtimeChunk) {
			Object.entries(options.entry).forEach(([entryName, value]) => {
				if (value.runtime === undefined) {
					value.runtime = runtimeChunk.name({ name: entryName });
				}
			});
		}
		// new EntryOptionPlugin().apply(compiler);
		assert(
			options.context,
			"options.context should have value after `applyRspackOptionsDefaults`"
		);
		compiler.hooks.entryOption.call(options.context, options.entry);

		new ModuleFederationRuntimePlugin().apply(compiler);

		const { minimize, minimizer } = options.optimization;
		if (minimize && minimizer) {
			for (const item of minimizer) {
				if (typeof item === "function") {
					(item as RspackPluginFunction).call(compiler, compiler);
				} else if (item !== "...") {
					item.apply(compiler);
				}
			}
		}

		if (options.optimization.mergeDuplicateChunks) {
			new MergeDuplicateChunksPlugin().apply(compiler);
		}

		if (options.builtins.devFriendlySplitChunks) {
			options.optimization.splitChunks = undefined;
		}

		if (
			options.optimization.splitChunks &&
			options.experiments.newSplitChunks === false
		) {
			new OldSplitChunksPlugin(options.optimization.splitChunks).apply(
				compiler
			);
		} else if (options.optimization.splitChunks) {
			new SplitChunksPlugin(options.optimization.splitChunks).apply(compiler);
		}

		if (options.optimization.nodeEnv) {
			new DefinePlugin({
				"process.env.NODE_ENV": JSON.stringify(options.optimization.nodeEnv)
			}).apply(compiler);
		}
		if (options.devServer?.hot) {
			options.output.strictModuleErrorHandling = true;
		}
		new ResolveSwcPlugin().apply(compiler);

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
