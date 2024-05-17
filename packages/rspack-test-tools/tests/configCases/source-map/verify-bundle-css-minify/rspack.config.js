/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			},
		},
	},
	devtool: "source-map",
	optimization: {
		minimize: true
	},
	externals: ["source-map"],
	externalsType: "commonjs"
};
