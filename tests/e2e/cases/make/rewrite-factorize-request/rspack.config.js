const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	context: __dirname,
	cache: true,
	experiments: {
		cache: true,
		incremental: true
	},
	plugins: [
		new rspack.HtmlRspackPlugin(),
		{
			apply(compiler) {
				let time = 0;
				compiler.hooks.compilation.tap(
					"PLUGIN",
					(_, { normalModuleFactory }) => {
						normalModuleFactory.hooks.resolve.tapPromise(
							"PLUGIN",
							async resolveData => {
								if (resolveData.request == "./file") {
									time++;
									resolveData.request = `./loader.js?time=${time}!./file`;
								}
							}
						);
					}
				);
			}
		}
	]
};
