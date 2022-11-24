module.exports = {
	entry: {
		main: ["./index.js"]
	},
	module: {
		rules: [
			{
				test: {
					type: "regexp",
					matcher: "\\.s[ac]ss$"
				},
				uses: [{ builtinLoader: "sass-loader" }],
				type: "css"
			}
		]
	}
};
