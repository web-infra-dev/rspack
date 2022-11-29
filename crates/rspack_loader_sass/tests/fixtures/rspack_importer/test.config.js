module.exports = {
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/,
				use: [{ builtinLoader: "sass-loader" }],
				type: "css"
			}
		]
	}
};
