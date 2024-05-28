const path = require("path");
const assert = require("assert");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: path.join(__dirname, "a.js"),
				use: [
					{
						loader: "./my-loader.js"
					}
				]
			},
		]
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.thisCompilation.tap("MyPlugin", compilation => {
					compilation.hooks.processAssets.tap("MyPlugin", () => {
						const chunk = compilation.chunks[0];
						const module = compilation.chunkGraph.getChunkModules(chunk);
						assert(module.buildInfo.LOADER_ACCESS === true);
						assert(module.buildMeta.LOADER_ACCESS === true);
					})
				})
			}
		}
	]
};
