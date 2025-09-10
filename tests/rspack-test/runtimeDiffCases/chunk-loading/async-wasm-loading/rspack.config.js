/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		wasmLoading: "async-node"
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
