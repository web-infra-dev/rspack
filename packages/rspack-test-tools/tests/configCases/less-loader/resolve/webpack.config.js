const path = require("path");
module.exports = {
	module: {
		rules: [
			{
				test: /\.less$/,
				use: [
					{
						loader: "less-loader",
						options: {
							lessOptions: {
								paths: ["node_modules", path.resolve(__dirname, "node_modules")]
							}
						}
					}
				],
				type: "css"
			}
		]
	}
};
