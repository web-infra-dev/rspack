const path = require("path");
const {
	experiments: { RstestPlugin }
} = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		entry: "./src/index.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		},
		output: {
			filename: "modulePathName.js"
		},
		plugins: [
			new RstestPlugin({
				injectModulePathName: true,
				hoistMockModule: true,
				importMetaPathName: true,
				manualMockRoot: path.resolve(__dirname, "__mocks__")
			})
		]
	},
	{
		entry: "./src/index.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		},
		output: {
			filename: "modulePathNameWithoutConcatenate.js"
		},
		plugins: [
			new RstestPlugin({
				injectModulePathName: true,
				hoistMockModule: true,
				importMetaPathName: true,
				manualMockRoot: path.resolve(__dirname, "__mocks__")
			})
		],
		optimization: {
			concatenateModules: false
		}
	},
	{
		entry: {
			main: "./index.js"
		},
		output: {
			filename: "[name].js"
		},
		externalsPresets: {
			node: true
		}
	}
];
