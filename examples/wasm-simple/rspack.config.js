/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	entry: {
		main: "./example.js"
	},
	mode: "development",
	output: {
		webassemblyModuleFilename: "[hash].wasm"
	},
	experiments: {
		asyncWebAssembly: true
	},
	builtins: {
		html: [{}]
	}
};
