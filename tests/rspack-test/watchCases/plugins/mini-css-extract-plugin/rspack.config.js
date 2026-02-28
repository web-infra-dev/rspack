var { CssExtractRspackPlugin: MCEP } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [MCEP.loader, "css-loader"],
				type: "javascript/auto",
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
