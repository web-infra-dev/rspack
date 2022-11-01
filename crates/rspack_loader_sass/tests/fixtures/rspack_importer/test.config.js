module.exports = {
	entry: {
		main: ["./index.js"]
	},
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
