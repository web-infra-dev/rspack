const path = require("path");
const HtmlRspackPlugin = require("html-rspack-plugin");

module.exports = [
	{
		plugins: [
			new HtmlRspackPlugin({
				filename: "html-index.html",
				template: "html-loader!" + path.join(__dirname, "template.html")
			})
		]
	},
	{
		plugins: [
			new HtmlRspackPlugin({
				filename: "pug-index.html",
				template:
					"@webdiscus/pug-loader!" + path.join(__dirname, "template.pug")
			})
		]
	}
];
