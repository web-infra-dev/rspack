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
						let hasModule = false;
						for (const chunk of compilation.chunks) {
							const modules = compilation.chunkGraph.getChunkModules(chunk);
							for (const module of modules) {
								if (module.identifier().endsWith("a.js")) {
									hasModule = true;
									assert(module.buildInfo.LOADER_ACCESS === true);
									assert(module.buildMeta.LOADER_ACCESS === true);
								}
							}
						}
						assert(hasModule);
					})
				})
			}
		}
	]
};
