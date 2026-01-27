const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	experiments: {
		cache: {
			type: "persistent"
		}
	},
	optimization: {
		providedExports: false
	},
	resolve: {
		alias: {
			data: [
				path.resolve(__dirname, "./data1.js"),
				path.resolve(__dirname, "./data2.js")
			]
		}
	}
};
