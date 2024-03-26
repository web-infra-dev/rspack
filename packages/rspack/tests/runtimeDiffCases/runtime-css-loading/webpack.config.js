const webpack = require("webpack");
const hmr = new webpack.HotModuleReplacementPlugin();
hmr.apply = hmr.apply.bind(hmr);

module.exports = {
	output: {
		publicPath: "/public/",
		cssFilename: "css/[name].css"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				loader: "css-loader",
				options: {
					modules: true
				}
			}
		]
	},
	experiments: {
		css: {
			exportsOnly: false
		}
	},
	plugins: [hmr]
};
