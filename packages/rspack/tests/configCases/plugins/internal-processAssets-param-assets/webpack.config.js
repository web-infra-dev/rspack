const { ConcatSource } = require("webpack-sources");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	entry: {
		main: "./index.js"
	},
	devtool: "source-map",
	plugins: [
		{
			name: "test",
			apply(compiler) {
				compiler.hooks.compilation.tap("compilation", compilation => {
					compilation.hooks.processAssets.tapPromise(
						{
							name: "processAssets1",
							stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE
						},
						async assets => {
							delete assets["main.js"];
						}
					);
				});
			}
		}
	]
};
