const path = require("path");
const HtmlPlugin = require("html-webpack-plugin");

module.exports = [
	{
		plugins: [
			new HtmlPlugin({
				filename: "html-index.html",
				template: "html-loader!" + path.join(__dirname, "template.html")
			})
		]
	},
	{
		plugins: [
			new HtmlPlugin({
				filename: "pug-index.html",
				template: "pug-loader!" + path.join(__dirname, "template.pug")
			})
		]
	}
];
