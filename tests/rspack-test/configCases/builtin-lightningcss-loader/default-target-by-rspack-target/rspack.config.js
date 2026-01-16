/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: ["web", "browserslist:chrome > 95"],
	module: {
		rules: [
			{
				test: /\.css$/,
				use: "builtin:lightningcss-loader",
			}
		]
	},
	node: {
		__dirname: false,
	}
};
