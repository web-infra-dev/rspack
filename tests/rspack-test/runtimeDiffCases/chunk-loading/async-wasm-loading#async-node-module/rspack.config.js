/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		wasmLoading: "async-node",
		module: true,
		environment: {
			dynamicImport: true,
			module: true
		}
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
