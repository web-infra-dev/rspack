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
		output: {
			library: {
				type: "commonjs"
			}
		},
		plugins: [
			new RslibPlugin({
				interceptApiPlugin: true
			})
		]
	},
	{
		entry: "./test.js",
		externals: {
			"./bundle0.js": "commonjs ./bundle0.js"
		},
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		}
	}
];
