import { RspackOptionsNormalized, Compiler } from ".";
import fs from "graceful-fs";

import { NodeTargetPlugin } from "./node/NodeTargetPlugin";
import { ResolveSwcPlugin } from "./web/ResolveSwcPlugin";
import { cleverMerge } from "./util/cleverMerge";
export class RspackOptionsApply {
	constructor() {}
	process(options: RspackOptionsNormalized, compiler: Compiler) {
		// @ts-expect-error
		compiler.outputPath = options.output.path;
		// @ts-expect-error
		compiler.name = options.name;
		compiler.outputFileSystem = fs;
		if (compiler.options.target.includes("node")) {
			new NodeTargetPlugin().apply(compiler);
		}
		// after we migrate minify to minimze, we could remove it
		// @ts-expect-error
		if (options.optimization.minimize || options.builtins.minify) {
			// @ts-expect-error
			if (options.optimization.minimizer) {
				// @ts-expect-error
				for (const minimizer of options.optimization.minimizer) {
					if (minimizer !== "...") {
						minimizer.apply(compiler);
					}
				}
			}
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
