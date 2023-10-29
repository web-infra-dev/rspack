const rspack = require("@rspack/core");
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
	plugins: [new rspack.HtmlRspackPlugin()]
};
module.exports = config;
