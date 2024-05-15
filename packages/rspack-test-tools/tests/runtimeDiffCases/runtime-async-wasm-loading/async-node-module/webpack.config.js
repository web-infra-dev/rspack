/** @type {import("webpack").Configuration} */
module.exports = {
	output: {
		wasmLoading: "async-node-module"
	},
	module: {
		rules: [
			{
				test: /\.wat$/,
				use: "wast-loader",
				type: "webassembly/async"
			}
		]
	},
	experiments: {
		asyncWebAssembly: true
	}
};
