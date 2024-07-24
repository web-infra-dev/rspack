/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: "source-map",
	optimization: {
		minimize: true,
		concatenateModules: false,
	},
	externals: ["source-map"],
	externalsType: "commonjs"
};
