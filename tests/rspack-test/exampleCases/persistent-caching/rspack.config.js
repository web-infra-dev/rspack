const path = require("path");
module.exports = {
	mode: "development",
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "javascript/auto",
				use: ["style-loader", "css-loader"]
			}
		]
	},
	experiments: {
		cache: {
			type: "persistent",
			buildDependencies: [__filename],
			storage: {
				type: "filesystem",
				directory: path.resolve(__dirname, ".cache")
			},
		}
	}
};
