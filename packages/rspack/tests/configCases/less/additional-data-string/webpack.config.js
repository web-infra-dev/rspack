const lessLoader = require("@rspack/less-loader");

module.exports = {
	module: {
		rules: [
			{
				test: /\.less$/,
				use: [
					{
						loader: lessLoader,
						options: {
							additionalData: "@background: coral;"
						}
					}
				],
				type: "css"
			}
		]
	}
};
