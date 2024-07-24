import type { Compiler, WebpackOptionsNormalized } from "webpack";

import { WebpackModulePlaceholderPlugin } from "./webpack-module-placeholder-plugin";

const PLUGIN_NAME = "WebpackDiffConfigPlugin";

export class WebpackDiffConfigPlugin {
	public name = PLUGIN_NAME;
	constructor(
		private modifier?: (
			options: WebpackOptionsNormalized
		) => WebpackOptionsNormalized
	) {}
	apply(compiler: Compiler) {
		const { options } = compiler;
		options.mode = "development";
		options.devtool = false;

		options.optimization ??= {};
		options.optimization.minimize = false;
		options.optimization.chunkIds = "named";
		options.optimization.moduleIds = "named";
		options.optimization.mangleExports = false;
		options.optimization.concatenateModules = false;

		options.output ??= {};
		options.output.pathinfo = false;

		options.output.environment ??= {};
		options.output.environment.arrowFunction ??= false;
		options.output.environment.bigIntLiteral ??= false;
		options.output.environment.const ??= false;
		options.output.environment.destructuring ??= false;
		options.output.environment.dynamicImport ??= false;
		options.output.environment.dynamicImportInWorker ??= false;
		options.output.environment.forOf ??= false;
		options.output.environment.globalThis ??= false;
		options.output.environment.module ??= false;
		options.output.environment.optionalChaining ??= false;
		options.output.environment.templateLiteral ??= false;

		if (typeof this.modifier === "function") {
			this.modifier(compiler.options);
		}

		new WebpackModulePlaceholderPlugin().apply(compiler);
	}
}
