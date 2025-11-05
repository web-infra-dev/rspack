const _rspack = require("../../packages/rspack/dist/index.js");

module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	mode: "development",
	devtool: false,
	output: {
		module: true,
		libraryTarget: "module",
		chunkFormat: "module"
	},
	optimization: {
		runtimeChunk: "single"
	},
	experiments: {
		mfAsyncStartup: false,
		outputModule: true
	},
	plugins: []
};
