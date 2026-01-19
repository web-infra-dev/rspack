const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: 'web',
	mode: "development",
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/resource"
			}
		]
	},
	experiments: {
		buildHttp: {
			allowedUris: ["https://"],
			lockfileLocation: path.resolve(__dirname, "./lock-files/lock.json"),
			cacheLocation: path.resolve(__dirname, "./lock-files/test")
		},
	}
};
