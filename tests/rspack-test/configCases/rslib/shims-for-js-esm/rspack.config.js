const {
	experiments: { RslibPlugin }
} = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		index: "./index.js",
	},
	target: "node",
	node: {
		__filename: 'node-module',
		__dirname: 'node-module'
	},
	experiments: {
		},
	optimization: {
		concatenateModules: false
	},
	module: {
		rules: [
			{
				// set every module type to javascript/esm
				type: 'javascript/esm',
			}
		]
	},
	output: {
		module: true,
		library: {
			type: "modern-module"
		},
		filename: 'bundle.mjs'
	},
	plugins: [
		new RslibPlugin({
			forceNodeShims: true
		})
	]
}
