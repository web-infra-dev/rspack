const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		entry1: "./index.js",
		entry2: "./index2.js",
		entry3: "./index3.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				styles: {
					name: "styles",
					chunks: "all",
					test: /\.css$/,
					enforce: true
				}
			}
		}
	},
	plugins: [
		new CssExtractRspackPlugin({
			ignoreOrder: false
		}),
		{
			apply(compiler) {
				compiler.hooks.done.tap("TestPlugin", stats => {
					const warnings = stats.toJson({ warnings: true }).warnings;
					expect(warnings.length).toBe(1);
					expect(warnings[0].file).toBe("styles.css")
				});
			}
		}
	]
};
