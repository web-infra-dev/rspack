module.exports = {
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
