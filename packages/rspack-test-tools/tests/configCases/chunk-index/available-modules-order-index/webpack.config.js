/** @typedef {import("../../../../types").Compilation} Compilation */
/** @typedef {import("../../../../types").Module} Module */
/** @type {import("../../../../types").Configuration} */
module.exports = {
	entry: {
		main: "./main.js"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: false,
		chunkIds: "named"
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
			}
		}
	},
	experiments: {
		css: true
	}
};
