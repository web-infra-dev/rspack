/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.png$/,
				generator: {
					filename: "[name][ext]"
				},
				type: "asset/resource"
			}
		]
	}
};
