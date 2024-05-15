const webpack = require("webpack");
const hmr = new webpack.HotModuleReplacementPlugin();
hmr.apply = hmr.apply.bind(hmr);

/** @type {import("webpack").Configuration} */
module.exports = {
	output: {
		chunkFilename: "[name].[fullhash].js"
	},
	plugins: [hmr]
};
