const {
	experiments: { RslibPlugin }
} = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		entry: "./index.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		},
		plugins: [
			new RslibPlugin({
				interceptApiPlugin: true
			})
		]
	},
	{
		entry: "./test.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		}
	}
];
