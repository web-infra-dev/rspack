const assert = require("assert");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	entry: {
		index: "./img.png"
	},
	mode: "development",
	output: {
		module: true,
		filename: `[name].js`,
		module: true,
		library: {
			type: "module"
		},
		iife: false,
		chunkFormat: "module",
		chunkLoading: "import"
	},
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/inline"
			}
		]
	},
	optimization: {
		concatenateModules: false,
		avoidEntryIife: true,
		minimize: false
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("MyPlugin", compilation => {
					compilation.hooks.processAssets.tap("MyPlugin", assets => {
						let list = Object.keys(assets);
						const js = list.find(item => item.endsWith("js"));
						const jsContent = assets[js].source().toString();

						expect(
							// should make sure no default property access for default ExportsType
							jsContent
						).toContain(
							"var __webpack_exports__default = __webpack_exports__;"
						);
					});
				});
			}
		}
	]
};
