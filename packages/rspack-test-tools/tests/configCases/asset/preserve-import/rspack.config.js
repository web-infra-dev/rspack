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
					importMode: "preserve"
				}
			}
		]
	},
	plugins: [
		new (class {
			apply(compiler) {
				compiler.hooks.compilation.tap("MyPlugin", compilation => {
					compilation.hooks.processAssets.tap("MyPlugin", assets => {
						let list = Object.keys(assets);
						const js = list.find(item => item.endsWith("js"));
						const jsContent = assets[js].source().toString();
						assert(/require\(['"]\.\/(\w*)\.png['"]\)/.test(jsContent));
					});
				});
			}
		})()
	]
};
