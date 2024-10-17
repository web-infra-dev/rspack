const { rspack } = require("@rspack/core");
var path = require("path");
/** @type {function(any, any): import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => [
	{
		entry: "./modern-module-non-entry-module-export.js",
		resolve: {
			alias: {
				library: path.resolve(testPath, "../0-create-library/modern-module-non-entry-module-export/main.js")
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("modern-module export from non-entry module")
			})
		]
	},
	{
		entry: "./modern-module-force-concatenation.js",
		resolve: {
			alias: {
				library: path.resolve(testPath, "../0-create-library/modern-module-force-concatenation")
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("modern-module force concatenation")
			})
		]
	},
];
