const { rspack } = require("@rspack/core");
const assert = require("assert");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	entry: {
		index: "./img.png"
	},
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/resource",
				generator: {
					importMode: "newURL"
				}
			}
		],
	},
	plugins: [
		new rspack.BannerPlugin({
			banner: 'var __import__meta__url__ = {};',
			stage: 0
		}),
		new rspack.DefinePlugin({
			'import.meta.url': '__import__meta__url__'
		}),
		new (class {
			apply(compiler) {
				compiler.hooks.compilation.tap("MyPlugin", compilation => {
					compilation.hooks.processAssets.tap("MyPlugin", assets => {
						let list = Object.keys(assets);
						const js = list.find(item => item.endsWith("js"));
						const jsContent = assets[js].source().toString();
												console.log(jsContent, 22222222)

						assert(/new URL\(['"]\.\/(\w*)\.png['"], import\.meta\.url\)/.test(jsContent));
					});
				});
			}
		})()
	]
};
