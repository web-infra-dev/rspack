module.exports = {
	mode: "development",
	entry: {
		main: "./src/main.js"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		],
		define: {
			__VUE_OPTIONS_API__: JSON.stringify(true),
			__VUE_PROD_DEVTOOLS__: JSON.stringify(false)
		}
	},
	devServer: {
		historyApiFallback: true
	},
	module: {
		rules: [
			{
				test: /\.vue$/,
				use: ["./vue-loader.js"]
			},
			{
				test: /\.vue$/,
				resourceQuery: /type=style/,
				use: ["./vue-loader.js"],
				type: "css"
			},
			{
				test: /\.svg/,
				type: "asset/resource"
			}
		]
	}
};
