const lessLoader = require("@rspack/plugin-less").default;

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
