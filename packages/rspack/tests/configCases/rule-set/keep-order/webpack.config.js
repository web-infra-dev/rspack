const path = require("path");
const resolve = filename => path.resolve(__dirname, filename);

module.exports = {
	entry: {
		main: "./index.js"
	},
	module: {
		rules: [
			{
				test: resolve("index.js"),
				use: [
					{
						loader: "./test-loader.js"
					}
				]
			}
		]
	}
};
