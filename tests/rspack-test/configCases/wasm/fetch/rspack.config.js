/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		target: "web",
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
			chunkFilename: "chunks/[name].async.mjs",
			webassemblyModuleFilename: "[id].[hash].module.async.wasm"
		},
		experiments: {
			asyncWebAssembly: true
		}
	},
	{
		target: "web",
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
			chunkFilename: "chunks/[name].async.js",
			webassemblyModuleFilename: "[id].[hash].async.wasm"
		},
		experiments: {
			asyncWebAssembly: true
		}
	},
];
