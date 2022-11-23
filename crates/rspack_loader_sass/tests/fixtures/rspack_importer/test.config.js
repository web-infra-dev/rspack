module.exports = {
	entry: {
		main: ["./index.js"]
	},
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				uses: [{ builtinLoader: "sass-loader" }],
				type: "css"
			}
		]
	}
};
