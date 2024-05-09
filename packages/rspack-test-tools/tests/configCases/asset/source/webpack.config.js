/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.txt$/,
				type: "asset/source"
			}
		]
	}
};
