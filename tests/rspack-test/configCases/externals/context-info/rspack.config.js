/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /other-layer\.js$/,
				layer: "other-layer"
			}
		]
	},
	externals: [
		function ({ context, request, contextInfo }, callback) {
			if (request === "external-pkg") {
				if (contextInfo.issuerLayer === "other-layer") {
					return callback(null, "var 2");
				}
				return callback(null, "var 1");
			}
			return callback();
		}
	],
};
