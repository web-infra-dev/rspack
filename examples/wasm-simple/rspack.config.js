/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	// mode: "development || "production",
	entry: {
		main: './example.js'
	},
	output: {
		webassemblyModuleFilename: "[hash].wasm",
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
		chunkIds: "deterministic" // To keep filename consistent between different modes (for example building only)
	},
	experiments: {
		asyncWebAssembly: true
	},
	builtins: {
		html: [
			{
				template: './index.html'
			}
		]
	}
};
