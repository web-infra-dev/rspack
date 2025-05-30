const { RstestPlugin } = require("@rspack/core");

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
				injectModulePathName: true
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
				injectModulePathName: true
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
