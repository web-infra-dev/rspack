module.exports = {
  output: {
    path: './dist',
  },
	module: {
		rules: [
			{
				test: "\\.s[ac]ss$",
				uses: [{ builtinLoader: "sass-loader" }],
				type: "css"
			}
		]
	},
}