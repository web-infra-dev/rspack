/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		target: "node",
		module: {
			rules: [
				{
					test: /\.wat$/,
					loader: "wast-loader",
					type: "webassembly/async"
				}
			]
		},
		output: {
			module: true,
			webassemblyModuleFilename: "[id].[hash].wasm"
		},
		experiments: {
			outputModule: true,
			asyncWebAssembly: true
		}
	},
	{
		target: "node",
		module: {
			rules: [
				{
					test: /\.wat$/,
					loader: "wast-loader",
					type: "webassembly/async"
				}
			]
		},
		output: {
			webassemblyModuleFilename: "[id].[hash].wasm"
		},
		experiments: {
			asyncWebAssembly: true
		}
	},
];
