module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: "\\.s[ac]ss$",
				uses: [{ builtinLoader: "sass-loader" }],
				type: "css"
			}
		]
	}
};
