const path = require("path");

const config = {
	entry: path.resolve(__dirname, "./index.js"),
	module: {
		rules: [
			{
				test: /\.(png|svg)$/i,
				type: "asset"
			}
		]
	}
};

module.exports = config;
