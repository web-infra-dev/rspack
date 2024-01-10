const { rspack } = require("@rspack/core");

module.exports = {
	output: {
		publicPath: "/base"
	},
	plugins: [new rspack.HtmlRspackPlugin({})]
};
