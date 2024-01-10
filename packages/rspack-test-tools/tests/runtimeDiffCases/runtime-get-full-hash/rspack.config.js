const webpack = require("@rspack/core");
const hmr = new webpack.HotModuleReplacementPlugin();
hmr.apply = hmr.apply.bind(hmr);

module.exports = {
	output: {
		chunkFilename: "[name].[fullhash].js"
	},
	plugins: [hmr]
};
