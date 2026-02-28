const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: ["node", "es2020"],
	output: {
		module: true,
		iife: true
	},
	optimization: {
		concatenateModules: false
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("Test", compilation => {
					compilation.hooks.processAssets.tap(
						{
							name: "copy-webpack-plugin",
							stage:
								compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
						},
						() => {
							compilation.emitAsset(
								"mod.js",
								new rspack.sources.RawSource(
									"module.exports = 'module text';\n"
								)
							);
						}
					);
				});
			}
		}
	]
};
