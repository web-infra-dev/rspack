const path = require("path");

/**@type {import('@rspack/core').Configuration} */
const config = {
	context: __dirname,
	mode: "development",
	output: {
		library: {
			type: "modern-module"
		},
		filename: "[name].js"
	},
	entry: {
		foo: "./foo.js",
		bar: "./bar.js"
	},
	externalsType: "module",
	externals: function ({ request }) {
		if (request.includes("value")) {
			// make '../modern-module-reexport-star/value' and './value'
			// the same request
			return path.resolve(__dirname, "./value.js");
		}
	},
	optimization: {
		avoidEntryIife: true,
		concatenateModules: true,
		minimize: false
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("MyPlugin", compilation => {
					compilation.hooks.processAssets.tap("MyPlugin", assets => {
						let list = Object.keys(assets);
						const js = list.find(item => item.includes("foo.js"));
						const jsContent = assets[js].source().toString();
						expect(
							// should make sure no default property access for default ExportsType
							jsContent
						).toContain("export * from");
					});
				});
			}
		}
	]
};

module.exports = config;
