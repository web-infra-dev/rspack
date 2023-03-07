module.exports = {
  output: {
    path: './dist',
  },
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [{ loader: "sass-loader" }],
				type: "css"
			}
		]
	},
	stats: "errors-warnings"
}