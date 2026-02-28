const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "css-loader", "./loader.js"],
				type: "javascript/auto"
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			chunkFilename: "bundle.css"
		}),
		compiler => {
			let initial = true;
			compiler.hooks.thisCompilation.tap("TEST_PLUGIN", compilation => {
				compilation.hooks.finishModules.tapAsync(
					"TEST_PLUGIN",
					(modules, callback) => {
						if (!initial) {
							return callback();
						}
						initial = false;
						const cssModule = Array.from(modules).find(
							module =>
								module.identifier().includes(".css") &&
								module.type === "javascript/auto"
						);
						compilation.rebuildModule(cssModule, callback);
					}
				);
			});
		}
	]
};
