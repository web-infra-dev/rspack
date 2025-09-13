module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	output: {
		webassemblyModuleFilename: "[name].wasm"
	},
	experiments: {
		asyncWebAssembly: true
	}
};
