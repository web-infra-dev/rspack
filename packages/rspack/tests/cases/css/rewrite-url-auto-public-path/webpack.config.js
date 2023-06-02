const { RawSource, ConcatSource } = require("webpack-sources");

module.exports = {
	entry: {
		main: "./index.js"
	},
	output: {
		publicPath: "auto",
		cssFilename: "css/[name].css"
	},
	resolve: {
		alias: {
			"@": __dirname
		}
	},
	module: {
		rules: [
			{
				test: /\.png$/i,
				type: "asset",
				generator: {
					filename: "image/[name][ext]"
				}
			}
		]
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("compilation", compilation => {
					compilation.hooks.processAssets.tapPromise(
						"polyfill for auto public path in target: 'node'",
						async assets => {
							compilation.updateAsset(
								"main.js",
								new ConcatSource(
									new RawSource(
										`self = { document: { currentScript: { src: "/" } } };\n`
									),
									assets["main.js"]
								)
							);
						}
					);
				});
			}
		}
	]
};
