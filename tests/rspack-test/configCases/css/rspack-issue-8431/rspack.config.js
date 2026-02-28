const { rspack } = require("@rspack/core");

class Plugin {
	apply(compiler) {
		compiler.hooks.thisCompilation.tap("MyPlugin", compilation => {
			compilation.hooks.processAssets.tap(
				{
					name: "MyPlugin",
					stage: compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_HASH
				},
				() => {
					const css = compilation
						.getAssets()
						.find(asset => asset.name.endsWith(".css"));
					expect(css.info.contenthash.length).toBeGreaterThan(0);
				}
			);
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	target: "web",
	module: {
		rules: [
			{
				test: /\.css$/,
				type: 'javascript/auto',
				use: [rspack.CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	plugins: [
		new rspack.CssExtractRspackPlugin({
			filename: "[name].[contenthash].css"
		}),
		new Plugin()
	],
	experiments: {
		css: false
	}
};
