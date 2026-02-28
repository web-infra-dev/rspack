const { HtmlRspackPlugin } = require("@rspack/core");

module.exports = [
	{
		plugins: [
			new HtmlRspackPlugin({
				filename: "body-index.html",
				inject: "body"
			})
		]
	},
	{
		plugins: [
			new HtmlRspackPlugin({
				filename: "head-index.html",
				inject: "head"
			})
		]
	},
	{
		plugins: [
			new HtmlRspackPlugin({
				filename: "true-blocking-index.html",
				inject: true,
				scriptLoading: "blocking"
			})
		]
	},
	{
		plugins: [
			new HtmlRspackPlugin({
				filename: "true-defer-index.html",
				inject: true
			})
		]
	},
	{
		plugins: [
			new HtmlRspackPlugin({
				filename: "false-index.html",
				inject: false
			})
		]
	}
];
