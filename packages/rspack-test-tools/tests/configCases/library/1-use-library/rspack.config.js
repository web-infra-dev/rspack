const { rspack } = require("@rspack/core");
var path = require("path");
/** @type {function(any, any): import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => [
	{
		entry: "./default-test.js",
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../0-create-library/modern-module-non-entry-module-export/main.js"
				)
			}
		},
		plugins: [
			new rspack.DefinePlugin({
				NAME: JSON.stringify("modern-module export from non-entry module")
			})
		]
	}
];
