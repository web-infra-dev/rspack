/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	optimization: {
		concatenateModules: true,
		minimize: false
	},
	module: {
		rules: [
			{
				test: /\.txt$/,
				type: "asset/resource"
			}
		]
	}
};
