/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: "source-map",
	optimization: {
		minimize: true
	},
	externals: ["source-map"],
	externalsType: "commonjs"
};
