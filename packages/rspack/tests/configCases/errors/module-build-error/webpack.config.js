module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [{ builtinLoader: "sass-loader" }],
				type: "css"
			}
		]
	}
};
