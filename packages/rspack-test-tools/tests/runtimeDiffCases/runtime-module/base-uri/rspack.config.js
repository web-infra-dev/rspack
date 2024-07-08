/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		bundle: {
			import: "./src/index.js",
			baseUri: "my-scheme://baseuri",
			publicPath: "/"
		}
	},
	output: {
		assetModuleFilename: "[name][ext]"
	},
	target: "web"
};
