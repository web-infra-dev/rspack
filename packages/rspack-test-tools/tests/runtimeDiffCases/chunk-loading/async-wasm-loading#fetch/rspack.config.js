/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		wasmLoading: "fetch"
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
