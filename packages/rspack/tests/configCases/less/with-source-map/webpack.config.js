const lessLoader = require("@rspack/less-loader");

module.exports = {
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.less$/,
				use: [{ loader: lessLoader }],
				type: "css"
			}
		]
	}
};
