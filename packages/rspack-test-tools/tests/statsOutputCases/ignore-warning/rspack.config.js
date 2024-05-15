/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./index",
	stats: "errors-warnings",
	ignoreWarnings: [/Using \/ for division outside/],
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [{ loader: "sass-loader" }],
				type: "css"
			}
		]
	}
};
