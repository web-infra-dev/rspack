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
	OptimizationRuntimeChunkNormalized
} from ".";
import fs from "graceful-fs";

import { ResolveSwcPlugin } from "./web/ResolveSwcPlugin";
import { DefaultStatsFactoryPlugin } from "./stats/DefaultStatsFactoryPlugin";
import { DefaultStatsPrinterPlugin } from "./stats/DefaultStatsPrinterPlugin";
import { cleverMerge } from "./util/cleverMerge";
import assert from "assert";
import IgnoreWarningsPlugin from "./lib/ignoreWarningsPlugin";
import EntryOptionPlugin from "./lib/EntryOptionPlugin";

export class RspackOptionsApply {
	constructor() {}
	process(options: RspackOptionsNormalized, compiler: Compiler) {
		assert(
			options.output.path,
			"options.output.path should have value after `applyRspackOptionsDefaults`"
		);
		assert(
			options.context,
			"options.context should have value after `applyRspackOptionsDefaults`"
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
		new EntryOptionPlugin().apply(compiler);
		compiler.hooks.entryOption.call(options.context, options.entry);

		const { minimize, minimizer } = options.optimization;
		if (minimize && minimizer) {
			for (const item of minimizer) {
				if (typeof item === "function") {
					item.call(compiler, compiler);
				} else if (item !== "...") {
					item.apply(compiler);
				}
			}
		}

		if (options.builtins.devFriendlySplitChunks) {
			options.optimization.splitChunks = undefined;
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
