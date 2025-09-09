/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.(png|svg|jpg)$/,
				type: "asset"
			}
		]
	}
};
