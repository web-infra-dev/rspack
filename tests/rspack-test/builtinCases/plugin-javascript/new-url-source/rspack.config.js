/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: "asset/resource"
			}
		]
	}
};
