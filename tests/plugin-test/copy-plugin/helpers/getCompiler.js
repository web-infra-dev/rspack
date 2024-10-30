const path = require("path");

const rspack = require("@rspack/core");

module.exports = (config = {}) => {
	const fullConfig = {
		mode: "development",
		context: path.resolve(__dirname, "../fixtures"),
		entry: path.resolve(__dirname, "../helpers/enter.js"),
		output: {
			path: path.resolve(__dirname, "../build")
		},
		module: {
			rules: [
				{
					test: /\.txt/,
					type: "asset/resource",
					generator: {
						filename: "asset-modules/[name][ext]"
					}
				}
			]
		},
		...config
	};

	const compiler = rspack(fullConfig);
	return compiler;
};
