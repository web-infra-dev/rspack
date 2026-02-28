const { rspack } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		output: {
			library: { type: "commonjs2" }
		},
		externals: {
			external: ["@rspack/core", "version"]
		},
		plugins: [
			new rspack.DefinePlugin({
				EXPECTED: JSON.stringify(rspack.version)
			})
		]
	},
	{
		externals: {
			external: ["Array", "isArray"]
		},
		plugins: [
			new rspack.DefinePlugin({
				EXPECTED: "Array.isArray"
			})
		]
	}
];
