import {
	Compiler,
	RspackOptionsNormalized,
	RspackPluginInstance
} from "@rspack/core";

const PLUGIN_NAME = "RspackDiffConfigPlugin";

export class RspackDiffConfigPlugin implements RspackPluginInstance {
	name = PLUGIN_NAME;

	constructor(
		private modifier?: (
			options: RspackOptionsNormalized
		) => RspackOptionsNormalized
	) {
		process.env["RSPACK_DIFF"] = "true"; // enable rspack diff
	}

	apply(compiler: Compiler) {
		const { options } = compiler;

		options.mode = "development";
		options.devtool = false;

		options.optimization ??= {};
		options.optimization.minimize = false;
		options.optimization.chunkIds = "named";
		options.optimization.moduleIds = "named";
		options.optimization.mangleExports = false;

		options.experiments ??= {};
		options.experiments.rspackFuture ??= {};

		if (typeof this.modifier === "function") {
			this.modifier(compiler.options);
		}
	}
}
