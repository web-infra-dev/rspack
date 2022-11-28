const lessLoader = require("@rspack/less-loader");

module.exports = {
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
