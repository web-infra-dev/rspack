import {
	RspackOptionsNormalized,
	Compiler,
	OptimizationRuntimeChunkNormalized
} from ".";
import fs from "graceful-fs";

import { NodeTargetPlugin } from "./node/NodeTargetPlugin";
import { ResolveSwcPlugin } from "./web/ResolveSwcPlugin";
import { cleverMerge } from "./util/cleverMerge";
import assert from "assert";

export class RspackOptionsApply {
	constructor() {}
	process(options: RspackOptionsNormalized, compiler: Compiler) {
		assert(
			options.output.path,
			"options.output.path should at least have a default value after `applyRspackOptionsDefaults`"
		);
		compiler.outputPath = options.output.path;
		compiler.name = options.name;
		compiler.outputFileSystem = fs;
		// TODO: align externalsPresets with webpack
		if (
			compiler.options.target !== false &&
			(compiler.options.target === "node" ||
				compiler.options.target?.includes("node"))
		) {
			new NodeTargetPlugin().apply(compiler);
		}

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
		const runtimeChunk = options.optimization
			.runtimeChunk as OptimizationRuntimeChunkNormalized;
		if (runtimeChunk) {
			Object.entries(options.entry).forEach(([entryName, value]) => {
				if (value.runtime === undefined) {
					value.runtime = runtimeChunk.name({ name: entryName });
				}
			});
		}
		if (options.builtins.devFriendlySplitChunks) {
			options.optimization.splitChunks = undefined;
		}
		if (options.devServer?.hot) {
			options.output.strictModuleErrorHandling = true;
		}
		new ResolveSwcPlugin().apply(compiler);

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
