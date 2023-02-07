module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [{ loader: "builtin:sass-loader" }],
				type: "css"
			}
		]
	}
};
