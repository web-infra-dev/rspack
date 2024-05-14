/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.tsx$/,
				use: {
					loader: "builtin:swc-loader",
				},
			}
		]
	}
};
