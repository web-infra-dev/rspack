/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	module: {
		rules: [
			{
				test: /\.js$/,
				type: "javascript/esm",
				use: "builtin:swc-loader"
			}
		]
	}
};
