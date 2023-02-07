module.exports = {
	mode: "development",
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
				use: [{ builtinLoader: "builtin:sass-loader" }],
				type: "css"
			}
		]
	}
};
