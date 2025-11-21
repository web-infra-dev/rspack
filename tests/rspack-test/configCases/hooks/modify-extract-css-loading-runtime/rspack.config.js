const webpack = require("@rspack/core");

class Plugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("TestFakePlugin", compilation => {
			compilation.hooks.runtimeModule.tap("TestFakePlugin", (module, chunk) => {
				if (module.constructorName === "CssLoadingRuntimeModule") {
					const originSource = module.source.source.toString("utf-8");
					module.source.source = Buffer.from(
						`${originSource}\n__webpack_require__.f.miniCss.test = true;\n`,
						"utf-8"
					);
				}
			});
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	target: "web",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [webpack.CssExtractRspackPlugin.loader, "css-loader"],
				type: "javascript/auto"
			}
		]
	},
	plugins: [new webpack.CssExtractRspackPlugin(), new Plugin()]
};
