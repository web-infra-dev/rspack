/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	entry: "./index.js",
	output: {
		filename: "bundle.js"
	},
	module: {
		rules: [
			{
				mimetype: "text/plain",
				type: "asset"
			}
		]
	},
	stats: { all: true }
};
