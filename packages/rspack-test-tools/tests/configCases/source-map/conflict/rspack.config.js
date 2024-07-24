/**
 * @type {import('webpack').Configuration | import('@rspack/cli').Configuration}
 */
module.exports = {
	mode: "development",
	devtool: "source-map", 
	externals: ["source-map"],
	entry: "./index.js",
};