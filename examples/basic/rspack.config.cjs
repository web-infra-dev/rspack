const path = require("node:path");

module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	mode: "development",
	devtool: false,
	module: {
		rules: [
			{
				test: /\.(png|jpg|jpeg|gif)$/i,
				type: "asset/resource"
			}
		]
	},
	experiments: {
		buildHttp: {
			allowedUris: [
				"https://raw.githubusercontent.com/",
				"https://github.com/"
			],
			cacheLocation: path.join(__dirname, "rspack-http-cache"),
			lockfileLocation: path.join(__dirname, "rspack-http-lockfile.json")
		}
	}
};
