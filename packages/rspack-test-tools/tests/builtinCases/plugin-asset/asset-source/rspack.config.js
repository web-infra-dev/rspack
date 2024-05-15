/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.txt$/,
				type: "asset/source"
			}
		]
	}
};
