/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	entry: {
		index: {
			import: ["./index.js"]
		}
	},
	output: {
		webassemblyModuleFilename: "[hash].wasm"
	},
	module: {
		rules: [
			{
				test: /\.wasm$/,
				type: "webassembly/async"
			}
		]
	},
	optimization: {
		minimize: false
	},
	experiments: {
		asyncWebAssembly: true
	}
};
