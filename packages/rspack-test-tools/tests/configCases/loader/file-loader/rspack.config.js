/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.png$/,
				use: [{ loader: "file-loader", options: { esModule: false } }],
				type: "js"
			}
		]
	}
};
