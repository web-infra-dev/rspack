const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	target: "web",
	output: {
		publicPath: "/",
		crossOriginLoading: "anonymous"
	},
	node: {
		__dirname: false
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			filename: "index.html"
		}),
		new rspack.SubresourceIntegrityPlugin({
			hashFuncNames: ["sha384"],
			enabled: true
		}),
		{
			apply(compiler) {
				compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
					rspack.HtmlRspackPlugin.getCompilationHooks(compilation).beforeAssetTagGeneration.tap('TestPlugin', (data) => {
						data.assets.js.push("https://cdn.jsdelivr.net/npm/react@18/umd/react.production.min.js");
					});
				});
			}
		}
	]
};
