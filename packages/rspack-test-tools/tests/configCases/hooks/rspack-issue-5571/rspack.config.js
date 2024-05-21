const rspack = require("@rspack/core");
const path = require("path");

class Plugin {
	apply(compiler) {
		compiler.hooks.thisCompilation.tap("TestFakePlugin", compilation => {
			compilation.hooks.runtimeModule.tap("TestFakePlugin", module => {
				if (module.name !== "css_loading" || !module.source) return;
				const originCode = module.source.source.toString("utf-8");

				module.source.source = Buffer.from(
					originCode
						.replace(
							/document\.(getElementsByTagName|querySelectorAll)/g,
							`(globalThis.APP_ROOT||document).$1`
						)
						.replace(
							/document\.head/g,
							`(globalThis.APP_STYLE_ROOT||document.head)`
						),
					"utf-8"
				);
			});
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	target: "web",
	mode: "development",
	node: {
		__dirname: false
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/module"
			}
		]
	},
	plugins: [new Plugin(), new rspack.HotModuleReplacementPlugin()]
};
