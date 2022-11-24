module.exports = {
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	module: {
		rules: [
			{
				test: {
					type: "regexp",
					matcher: "\\.s[ac]ss$"
				},
				use: [{ builtinLoader: "sass-loader" }],
				type: "css"
			}
		]
	}
};
