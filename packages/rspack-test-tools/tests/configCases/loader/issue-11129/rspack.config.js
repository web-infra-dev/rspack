const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: path.resolve(__dirname, "lib.js"),
				resourceQuery: /inline/,
				use: "exports-loader?type=commonjs&exports=single|lamejs"
			},
			{
				test: path.resolve(__dirname, "lib.js"),
				resourceQuery: /object/,
				use: {
					loader: "exports-loader",
					options: {
						type: "commonjs",
						exports: "single|lamejs"
					}
				}
			}
		]
	}
};
