var MCEP = require("mini-css-extract-plugin");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [MCEP.loader, "css-loader"]
			}
		]
	},
	output: {
		publicPath: ""
	},
	target: "web",
	node: {
		__dirname: false
	},
	plugins: [new MCEP()]
};
