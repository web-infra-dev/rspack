/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.png$/,
				use: [{ loader: "./loader.js", parallel: true, options: {} }],
				type: "asset/resource"
			}
		]
	},
};
