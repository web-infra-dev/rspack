const {
	experiments: { RstestPlugin }
} = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		entry: "./index.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		},
		output: {
			filename: "bundle.js"
		},
		plugins: [
			new RstestPlugin({
				injectModulePathName: true,
				hoistMockModule: true,
				importMetaPathName: true,
				manualMockRoot: __dirname,
				preserveNewUrl: [".wasm"]
			})
		]
	},
	{
		entry: {
			main: "./test.js"
		},
		output: {
			filename: "[name].js"
		},
		externalsPresets: {
			node: true
		}
	}
];
