/** @type {import('@rspack/cli').Configuration} */
const config = {
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
module.exports = config;
