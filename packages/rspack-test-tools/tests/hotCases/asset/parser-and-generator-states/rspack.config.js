/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.(svg|png)$/,
				type: "asset"
			}
		]
	}
};
