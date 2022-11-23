module.exports = {
  output: {
    path: './dist',
  },
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				uses: [{ builtinLoader: "sass-loader" }],
				type: "css"
			}
		]
	},
}