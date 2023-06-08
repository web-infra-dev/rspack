const { VueLoaderPlugin } = require("vue-loader");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
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
	plugins: [new VueLoaderPlugin()],
	module: {
		rules: [
			{
				test: /\.vue$/,
				loader: "vue-loader"
			},
			{
				test: /\.less$/,
				use: [
					{
						loader: "style-loader",
						options: {
							esModule: false
						}
					},
					"css-loader",
					"less-loader"
				],
				type: "javascript/auto"
			},
			{
				test: /\.css$/,
				use: [
					{
						loader: "style-loader",
						options: {
							esModule: false
						}
					},
					"css-loader"
				],
				type: "javascript/auto"
			},
			{
				test: /\.svg/,
				type: "asset/resource"
			}
		]
	},
	experiments: {
		css: false
	}
};
module.exports = config;
